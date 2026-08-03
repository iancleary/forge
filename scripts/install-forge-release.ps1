[CmdletBinding()]
param(
    [string]$Tag = $env:FORGE_TAG,
    [switch]$SkipCodex,
    [switch]$VerifyAttestation,
    [switch]$BuildFromSource
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ForgeRepoSlug = "iancleary/forge"
$ForgeRepoUrl = "https://github.com/$ForgeRepoSlug"
$ForgeApiUrl = "https://api.github.com/repos/$ForgeRepoSlug"
$ForgeRawUrl = "https://raw.githubusercontent.com/$ForgeRepoSlug"
$ForgeDownloadUrl = "$ForgeRepoUrl/releases/download"
$ForgeBinaryNames = @(
    "forge.exe"
    "codex-threads.exe"
    "linear.exe"
    "mermaid.exe"
    "slack-agent.exe"
    "slack-query.exe"
)
$ForgeReleaseWorkflow = "$ForgeRepoSlug/.github/workflows/release-artifacts.yml"

function Fail([string]$Message) {
    throw "error: $Message"
}

function Invoke-ForgeDownload([string]$Uri, [string]$Destination) {
    $parsed = [Uri]$Uri
    if ($parsed.Scheme -ne "https" -or
        ($parsed.Host -notin @("github.com", "api.github.com", "raw.githubusercontent.com"))) {
        Fail "refusing download from unexpected URL: $Uri"
    }

    $fixtureRoot = $env:FORGE_TEST_FIXTURE_ROOT
    if (-not [string]::IsNullOrWhiteSpace($fixtureRoot)) {
        if ($Uri -eq "$ForgeApiUrl/releases/latest") {
            $fixtureName = "releases-latest.json"
        } else {
            $fixtureName = ($parsed.AbsolutePath.TrimEnd("/") -split "/")[-1]
        }
        $fixturePath = Join-Path $fixtureRoot $fixtureName
        if (-not (Test-Path -LiteralPath $fixturePath -PathType Leaf)) {
            Fail "fixture response is missing: $fixturePath"
        }
        Copy-Item -LiteralPath $fixturePath -Destination $Destination -Force
        return
    }

    for ($attempt = 1; $attempt -le 3; $attempt++) {
        try {
            Invoke-WebRequest -Uri $Uri -OutFile $Destination -UseBasicParsing -ErrorAction Stop
            return
        } catch {
            if ($attempt -eq 3) {
                Fail "download failed for $Uri after $attempt attempts: $($_.Exception.Message)"
            }
            Start-Sleep -Seconds $attempt
        }
    }
}

function Resolve-ForgeTag {
    if (-not [string]::IsNullOrWhiteSpace($Tag)) {
        return
    }
    $latestPath = Join-Path ([IO.Path]::GetTempPath()) ("forge-latest-" + [Guid]::NewGuid().ToString("N") + ".json")
    try {
        Invoke-ForgeDownload "$ForgeApiUrl/releases/latest" $latestPath
        $latest = Get-Content -LiteralPath $latestPath -Raw | ConvertFrom-Json
        $script:Tag = [string]$latest.tag_name
    } finally {
        Remove-Item -LiteralPath $latestPath -Force -ErrorAction SilentlyContinue
    }
    if ([string]::IsNullOrWhiteSpace($Tag)) {
        Fail "failed to resolve the latest Forge release tag"
    }
}

function Validate-ForgeTag {
    if ($Tag -notmatch '^[0-9]{8}\.0\.[0-9]+$') {
        Fail "invalid Forge release tag: $Tag"
    }
}

function Invoke-TaggedInstaller {
    if ($env:FORGE_INSTALLER_PINNED -eq "1") {
        return
    }
    $taggedPath = Join-Path ([IO.Path]::GetTempPath()) ("install-forge-release-" + [Guid]::NewGuid().ToString("N") + ".ps1")
    try {
        Invoke-ForgeDownload "$ForgeRawUrl/$Tag/scripts/install-forge-release.ps1" $taggedPath
        $forward = @("-Tag", $Tag)
        if ($SkipCodex) { $forward += "-SkipCodex" }
        if ($VerifyAttestation) { $forward += "-VerifyAttestation" }
        if ($BuildFromSource) { $forward += "-BuildFromSource" }
        $previous = $env:FORGE_INSTALLER_PINNED
        $env:FORGE_INSTALLER_PINNED = "1"
        try {
            & $taggedPath @forward
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        } finally {
            $env:FORGE_INSTALLER_PINNED = $previous
        }
    } finally {
        Remove-Item -LiteralPath $taggedPath -Force -ErrorAction SilentlyContinue
    }
    exit 0
}

function Get-ForgeChecksum([string]$ManifestPath, [string]$AssetName) {
    $records = @{}
    foreach ($line in (Get-Content -LiteralPath $ManifestPath)) {
        if ([string]::IsNullOrEmpty($line)) { continue }
        if ($line -notmatch '^(?<hash>[0-9A-Fa-f]{64})  (?<name>[^\s]+)$') {
            Fail "malformed checksum manifest entry"
        }
        $name = $Matches.name
        if ($records.ContainsKey($name)) {
            Fail "duplicate checksum manifest entry: $name"
        }
        $records[$name] = $Matches.hash.ToLowerInvariant()
    }
    if (-not $records.ContainsKey($AssetName)) {
        Fail "missing checksum entry for $AssetName"
    }
    return $records[$AssetName]
}

function Test-ReparsePoint([string]$Path) {
    $item = $null
    try {
        $fileSystemInfo = [IO.DirectoryInfo]::new($Path)
        if (-not [string]::IsNullOrWhiteSpace([string]$fileSystemInfo.LinkTarget) -or
            $null -ne $fileSystemInfo.ResolveLinkTarget($false)) {
            return $true
        }
    } catch {
        # Fall through to the provider and attribute checks.
    }
    try {
        $parentPath = Split-Path -LiteralPath $Path -Parent
        $leafName = Split-Path -LiteralPath $Path -Leaf
        $item = Get-ChildItem -LiteralPath $parentPath -Force -ErrorAction Stop |
            Where-Object { $_.Name -eq $leafName } |
            Select-Object -First 1
        if ($null -ne $item) {
            foreach ($propertyName in @("LinkType", "Target", "LinkTarget")) {
                $property = $item.PSObject.Properties[$propertyName]
                if ($null -ne $property -and
                    -not [string]::IsNullOrWhiteSpace([string]$property.Value)) {
                    return $true
                }
            }
            $attributesProperty = $item.PSObject.Properties["Attributes"]
            if ($null -ne $attributesProperty -and
                (($attributesProperty.Value -band [IO.FileAttributes]::ReparsePoint) -ne 0)) {
                return $true
            }
        }
    } catch {
        # Fall through to the .NET and provider metadata checks.
    }
    try {
        $attributes = [IO.File]::GetAttributes($Path)
        if (($attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            return $true
        }
    } catch {
        # Broken links may not be readable through the .NET attribute API.
    }
    $item = Get-Item -LiteralPath $Path -Force -ErrorAction SilentlyContinue
    if ($null -eq $item) { return $false }
    foreach ($propertyName in @("LinkType", "Target", "LinkTarget")) {
        $property = $item.PSObject.Properties[$propertyName]
        if ($null -ne $property -and
            -not [string]::IsNullOrWhiteSpace([string]$property.Value)) {
            return $true
        }
    }
    $attributesProperty = $item.PSObject.Properties["Attributes"]
    return ($null -ne $attributesProperty -and
        (($attributesProperty.Value -band [IO.FileAttributes]::ReparsePoint) -ne 0))
}

function Get-ForgeDestination {
    if ([string]::IsNullOrWhiteSpace($env:LOCALAPPDATA)) {
        Fail "LOCALAPPDATA is not set"
    }
    $destination = Join-Path $env:LOCALAPPDATA "Forge\bin"
    if (Test-ReparsePoint $destination) {
        Fail "refusing symlink destination directory: $destination"
    }
    if (Test-Path -LiteralPath $destination -PathType Leaf) {
        Fail "destination is not a directory: $destination"
    }
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    return $destination
}

function Assert-RegularFile([string]$Path) {
    if (Test-ReparsePoint $Path) {
        Fail "refusing reparse-point binary: $Path"
    }
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Fail "expected regular binary: $Path"
    }
    if ((Get-Item -LiteralPath $Path).Length -le 0) {
        Fail "binary is empty: $Path"
    }
}

function Expand-ForgeZip([string]$ArchivePath, [string]$Destination) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $expected = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($name in $ForgeBinaryNames) { [void]$expected.Add($name) }
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $zip = [IO.Compression.ZipFile]::OpenRead($ArchivePath)
    try {
        foreach ($entry in $zip.Entries) {
            $name = $entry.FullName
            if ($name.EndsWith("/")) { Fail "release archive contains a directory: $name" }
            if ($name -match '(^/|\\|(^|/)\.\.?(/|$))') {
                Fail "release archive contains an unsafe path: $name"
            }
            if (-not $expected.Contains($name)) { Fail "release archive contains unexpected entry: $name" }
            if (-not $seen.Add($name)) { Fail "release archive contains duplicate entry: $name" }
            if ($entry.PSObject.Properties.Name -contains "ExternalAttributes") {
                $mode = ([uint32]$entry.ExternalAttributes) -shr 16
                if (($mode -band 0xF000) -eq 0xA000) { Fail "release archive contains a symbolic link: $name" }
            }
        }
        if ($seen.Count -ne $expected.Count) { Fail "release archive must contain exactly six entries" }
        foreach ($name in $ForgeBinaryNames) {
            if (-not $seen.Contains($name)) { Fail "release archive is missing binary: $name" }
        }

        New-Item -ItemType Directory -Path $Destination -Force | Out-Null
        foreach ($entry in $zip.Entries) {
            $outPath = Join-Path $Destination $entry.FullName
            $input = $entry.Open()
            try {
                $output = [IO.File]::Open($outPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
                try { $input.CopyTo($output) } finally { $output.Dispose() }
            } finally { $input.Dispose() }
            Assert-RegularFile $outPath
        }
    } finally {
        $zip.Dispose()
    }
}

function Install-ForgeCandidateSet([string]$SourceDirectory) {
    $destination = Get-ForgeDestination
    $stage = Join-Path $destination (".forge-stage-" + [Guid]::NewGuid().ToString("N"))
    $backup = Join-Path $destination (".forge-backup-" + [Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $stage, $backup -Force | Out-Null
    $moved = [Collections.Generic.List[string]]::new()
    $installed = [Collections.Generic.List[string]]::new()
    $completed = $false
    try {
        foreach ($name in $ForgeBinaryNames) {
            $source = Join-Path $SourceDirectory $name
            Assert-RegularFile $source
            $staged = Join-Path $stage $name
            [IO.File]::Copy($source, $staged, $false)
            Assert-RegularFile $staged
        }
        foreach ($name in $ForgeBinaryNames) {
            $target = Join-Path $destination $name
            if (Test-ReparsePoint $target) { Fail "refusing symlink binary destination: $target" }
            if (Test-Path -LiteralPath $target -PathType Container) { Fail "existing binary destination is a directory: $target" }
            if (Test-Path -LiteralPath $target -PathType Leaf) {
                $backupPath = Join-Path $backup $name
                [IO.File]::Move($target, $backupPath)
                [void]$moved.Add($name)
            }
        }
        $replaceCount = 0
        foreach ($name in $ForgeBinaryNames) {
            [IO.File]::Move((Join-Path $stage $name), (Join-Path $destination $name))
            [void]$installed.Add($name)
            $replaceCount++
            if ($env:FORGE_TEST_FAIL_REPLACEMENT_AFTER -and
                [int]$env:FORGE_TEST_FAIL_REPLACEMENT_AFTER -eq $replaceCount) {
                Fail "test replacement failure"
            }
        }
        $completed = $true
    } catch {
        $rollbackFailed = $false
        foreach ($name in $installed) {
            try { Remove-Item -LiteralPath (Join-Path $destination $name) -Force } catch { $rollbackFailed = $true }
        }
        foreach ($name in $moved) {
            try { [IO.File]::Move((Join-Path $backup $name), (Join-Path $destination $name)) } catch { $rollbackFailed = $true }
        }
        if ($rollbackFailed) {
            Fail "replacement failed and rollback failed; recovery files remain in $backup"
        }
        $completed = $true
        throw
    } finally {
        if ($completed) {
            Remove-Item -LiteralPath $stage, $backup -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

function Install-ForgeArtifact {
    $assetName = "forge-$Tag-x86_64-pc-windows-msvc.zip"
    $temp = Join-Path ([IO.Path]::GetTempPath()) ("forge-install-" + [Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $temp -Force | Out-Null
    try {
        $manifest = Join-Path $temp "forge-release-sha256sums.txt"
        Invoke-ForgeDownload "$ForgeDownloadUrl/$Tag/forge-release-sha256sums.txt" $manifest
        $expected = Get-ForgeChecksum $manifest $assetName
        $archive = Join-Path $temp $assetName
        Invoke-ForgeDownload "$ForgeDownloadUrl/$Tag/$assetName" $archive
        $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $expected) { Fail "checksum mismatch for ${assetName}: expected $expected, got $actual" }
        if ($VerifyAttestation) {
            if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { Fail "gh is required for explicit attestation verification" }
            & gh attestation verify $archive --repo $ForgeRepoSlug --source-ref "refs/tags/$Tag" --signer-workflow $ForgeReleaseWorkflow --predicate-type "https://slsa.dev/provenance/v1"
            if ($LASTEXITCODE -ne 0) { Fail "explicit GitHub attestation verification failed for $assetName" }
        }
        $extract = Join-Path $temp "extract"
        Expand-ForgeZip $archive $extract
        Install-ForgeCandidateSet $extract
    } finally {
        Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Install-ForgeSource {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Fail "missing required command: cargo" }
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) { Fail "missing required command: git" }
    $temp = Join-Path ([IO.Path]::GetTempPath()) ("forge-source-" + [Guid]::NewGuid().ToString("N"))
    $repo = Join-Path $temp "repo"
    New-Item -ItemType Directory -Path $temp -Force | Out-Null
    try {
        & git clone --depth 1 --branch $Tag $ForgeRepoUrl $repo
        if ($LASTEXITCODE -ne 0) { Fail "failed to clone $ForgeRepoUrl at tag $Tag" }
        $cargoArgs = @("build", "--release", "--locked")
        foreach ($name in $ForgeBinaryNames) {
            $binary = $name.Substring(0, $name.Length - 4)
            $cargoArgs += @("-p", $binary, "--bin", $binary)
        }
        Push-Location $repo
        try { & cargo @cargoArgs } finally { Pop-Location }
        if ($LASTEXITCODE -ne 0) { Fail "tagged source build failed" }
        Install-ForgeCandidateSet (Join-Path $repo "target\release")
    } finally {
        Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

try {
    if (-not [Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([Runtime.InteropServices.OSPlatform]::Windows)) {
        Fail "this installer supports native Windows x64 only; use the POSIX installer for Linux or macOS"
    }
    if ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne [Runtime.InteropServices.Architecture]::X64) {
        Fail "this installer supports native Windows x64 only; Windows ARM and 32-bit Windows are not supported"
    }
    if ($VerifyAttestation -and $BuildFromSource) {
        Fail "-VerifyAttestation cannot be combined with -BuildFromSource"
    }
    Resolve-ForgeTag
    Validate-ForgeTag
    Invoke-TaggedInstaller
    if ($BuildFromSource) { Install-ForgeSource } else { Install-ForgeArtifact }

    $forge = Join-Path (Get-ForgeDestination) "forge.exe"
    if (-not $env:FORGE_TEST_SKIP_ASSETS) {
        & $forge skills install --all --target user
        if ($LASTEXITCODE -ne 0) { Fail "Forge skill reconciliation failed" }
        if (-not $SkipCodex) {
            & $forge codex install
            if ($LASTEXITCODE -ne 0) { Fail "Forge Codex reconciliation failed" }
        }
    }
    Write-Host "Forge installed to $(Get-ForgeDestination)."
    Write-Host "Add that directory to your user PATH if it is not already present; the installer does not modify PATH."
} catch {
    Write-Error $_.Exception.Message
    exit 1
}
