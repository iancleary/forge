Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
$Version = "20260802.0.0"
$AssetName = "forge-$Version-x86_64-pc-windows-msvc.zip"
$TestRoot = Join-Path ([IO.Path]::GetTempPath()) ("forge-installer-test-" + [Guid]::NewGuid().ToString("N"))
$FakeBin = Join-Path $TestRoot "fake-bin"
New-Item -ItemType Directory -Path $FakeBin -Force | Out-Null
$OriginalPath = $env:Path
$OriginalLocalAppData = $env:LOCALAPPDATA
Add-Type -AssemblyName System.IO.Compression.FileSystem

function Fail([string]$Message) { throw "installer fixture test failed: $Message" }

function Get-Sha256([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function New-Zip([string]$Files, [string]$ArchivePath, [string]$Mode) {
    $stream = [IO.File]::Create($ArchivePath)
    $archive = [IO.Compression.ZipArchive]::new($stream, [IO.Compression.ZipArchiveMode]::Create)
    try {
        $names = @("forge.exe", "codex-threads.exe", "linear.exe", "mermaid.exe", "slack-agent.exe", "slack-query.exe")
        if ($Mode -eq "missing") { $names = $names[0..4] }
        if ($Mode -eq "unexpected") { $names += "extra.exe"; Set-Content -LiteralPath (Join-Path $Files "extra.exe") -Value "extra" }
        foreach ($name in $names) {
            $source = Join-Path $Files $name
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $source, $name) | Out-Null
        }
        if ($Mode -eq "duplicate") {
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, (Join-Path $Files "forge.exe"), "forge.exe") | Out-Null
        }
        if ($Mode -eq "traversal") {
            $entry = $archive.CreateEntry("../escape.exe")
            $writer = [IO.StreamWriter]::new($entry.Open())
            try { $writer.Write("escape") } finally { $writer.Dispose() }
        }
    } finally {
        $archive.Dispose()
        $stream.Dispose()
    }
}

function New-Fixture([string]$Label, [string]$ArchiveMode = "valid", [string]$ChecksumMode = "valid") {
    $fixture = Join-Path $TestRoot $Label
    $files = Join-Path $fixture "files"
    New-Item -ItemType Directory -Path $files -Force | Out-Null
    foreach ($name in @("forge.exe", "codex-threads.exe", "linear.exe", "mermaid.exe", "slack-agent.exe", "slack-query.exe")) {
        Set-Content -LiteralPath (Join-Path $files $name) -Value "fixture $Label"
    }
    $archive = Join-Path $fixture $AssetName
    New-Zip $files $archive $ArchiveMode
    $hash = Get-Sha256 $archive
    switch ($ChecksumMode) {
        "valid" { "$hash  $AssetName" | Set-Content -LiteralPath (Join-Path $fixture "forge-release-sha256sums.txt") }
        "mismatch" { ("0" * 64) + "  " + $AssetName | Set-Content -LiteralPath (Join-Path $fixture "forge-release-sha256sums.txt") }
        "missing" { ("0" * 64) + "  other.zip" | Set-Content -LiteralPath (Join-Path $fixture "forge-release-sha256sums.txt") }
        "duplicate" { @("$hash  $AssetName", "$hash  $AssetName") | Set-Content -LiteralPath (Join-Path $fixture "forge-release-sha256sums.txt") }
        "malformed" { "not a checksum record" | Set-Content -LiteralPath (Join-Path $fixture "forge-release-sha256sums.txt") }
        default { Fail "unknown checksum mode: $ChecksumMode" }
    }
    return $fixture
}

function Run-Installer([string]$Fixture, [string[]]$Arguments = @(), [string]$TestHome = $null, [switch]$DestinationReparsePoint) {
    if ([string]::IsNullOrWhiteSpace($TestHome)) { $TestHome = Join-Path $TestRoot ("home-" + [Guid]::NewGuid().ToString("N")) }
    $localAppData = Join-Path $TestHome "localappdata"
    New-Item -ItemType Directory -Path $localAppData -Force | Out-Null
    if ($DestinationReparsePoint) {
        $forgeDirectory = Join-Path $localAppData "Forge"
        $redirect = Join-Path $TestHome "redirect"
        New-Item -ItemType Directory -Path $forgeDirectory, $redirect -Force | Out-Null
        $destinationLink = Join-Path $forgeDirectory "bin"
        New-Item -ItemType SymbolicLink -Path $destinationLink -Target $redirect | Out-Null
    }
    $log = Join-Path $TestHome "tool.log"
    $ghLog = Join-Path $TestHome "gh.log"
    New-Item -ItemType File -Path $log, $ghLog -Force | Out-Null
    $env:LOCALAPPDATA = $localAppData
    $env:FORGE_INSTALLER_PINNED = "1"
    $env:FORGE_TEST_FIXTURE_ROOT = $Fixture
    $env:FORGE_TEST_SKIP_ASSETS = "1"
    $env:FORGE_TEST_TOOL_LOG = $log
    $env:FORGE_TEST_GH_LOG = $ghLog
    $env:FORGE_TEST_GH_MODE = if ($env:FORGE_TEST_GH_MODE) { $env:FORGE_TEST_GH_MODE } else { "fail" }
    $env:Path = "$FakeBin;$OriginalPath"
    try {
        $global:LASTEXITCODE = 0
        $installerParameters = @{ Tag = $Version; SkipCodex = $true }
        foreach ($argument in $Arguments) {
            switch ($argument) {
                "-VerifyAttestation" { $installerParameters.VerifyAttestation = $true }
                "-BuildFromSource" { $installerParameters.BuildFromSource = $true }
                default { throw "unsupported installer fixture argument: $argument" }
            }
        }
        & $Root/scripts/install-forge-release.ps1 @installerParameters
        if ($LASTEXITCODE -ne 0) { throw "installer exited with $LASTEXITCODE" }
    } catch {
        return @{ Success = $false; Home = $TestHome; Log = $log; GhLog = $ghLog; Error = $_.Exception.Message }
    }
    return @{ Success = $true; Home = $TestHome; Log = $log; GhLog = $ghLog; Error = $null }
}

function Expect-Failure([string]$Fixture, [string[]]$Arguments = @(), [string]$TestHome = $null) {
    $result = Run-Installer $Fixture $Arguments $TestHome
    if ($result.Success) { Fail "expected installer failure for $Fixture" }
    return $result
}

try {
    @"
@echo off
echo %*>>%FORGE_TEST_GH_LOG%
if "%FORGE_TEST_GH_MODE%"=="success" exit /b 0
exit /b 1
"@ | Set-Content -Encoding ascii -LiteralPath (Join-Path $FakeBin "gh.cmd")
    @"
@echo off
echo git %*>>%FORGE_TEST_TOOL_LOG%
exit /b 1
"@ | Set-Content -Encoding ascii -LiteralPath (Join-Path $FakeBin "git.cmd")
    @"
@echo off
echo cargo %*>>%FORGE_TEST_TOOL_LOG%
exit /b 1
"@ | Set-Content -Encoding ascii -LiteralPath (Join-Path $FakeBin "cargo.cmd")

    $valid = New-Fixture "valid"
    $result = Run-Installer $valid
    if (-not $result.Success) { Fail "valid Windows install failed: $($result.Error)" }
    if (-not (Test-Path -LiteralPath (Join-Path $result.Home "localappdata\Forge\bin\forge.exe") -PathType Leaf)) { Fail "valid Windows install did not install forge.exe" }
    if (-not [string]::IsNullOrWhiteSpace((Get-Content -LiteralPath $result.Log -Raw))) { Fail "default Windows install invoked a toolchain command" }
    if (-not [string]::IsNullOrWhiteSpace((Get-Content -LiteralPath $result.GhLog -Raw))) { Fail "default Windows install invoked GitHub CLI" }

    $junctionFixtureReady = $false
    try {
        $junctionProbeHome = Join-Path $TestRoot ("junction-probe-" + [Guid]::NewGuid().ToString("N"))
        $junctionProbeAppData = Join-Path $junctionProbeHome "localappdata"
        $junctionProbeForge = Join-Path $junctionProbeAppData "Forge"
        New-Item -ItemType Directory -Path $junctionProbeForge, (Join-Path $junctionProbeHome "redirect") -Force | Out-Null
        $junctionProbePath = Join-Path $junctionProbeForge "bin"
        New-Item -ItemType SymbolicLink -Path $junctionProbePath -Target (Join-Path $junctionProbeHome "redirect") | Out-Null
        $junctionFixtureReady = $true
    } catch {
        Write-Host "reparse-point destination fixture unavailable; native Windows installer contract remains covered by the POSIX fixture"
    }
    if ($junctionFixtureReady) {
        $junctionResult = Run-Installer -Fixture $valid -Arguments @() -DestinationReparsePoint
        if ($junctionResult.Success) { Fail "reparse-point destination was accepted" }
    }

    foreach ($mode in @("mismatch", "missing", "duplicate", "malformed")) {
        [void](Expect-Failure (New-Fixture "checksum-$mode" "valid" $mode))
    }
    foreach ($mode in @("missing", "unexpected", "duplicate", "traversal")) {
        [void](Expect-Failure (New-Fixture "archive-$mode" $mode))
    }

    $result = Run-Installer (New-Fixture "rollback")
    if (-not $result.Success) { Fail "rollback setup failed: $($result.Error)" }
    $destination = Join-Path $result.Home "localappdata\Forge\bin"
    foreach ($name in @("forge.exe", "codex-threads.exe", "linear.exe", "mermaid.exe", "slack-agent.exe", "slack-query.exe")) {
        Set-Content -LiteralPath (Join-Path $destination $name) -Value "old-$name"
    }
    $env:FORGE_TEST_FAIL_REPLACEMENT_AFTER = "1"
    $rollback = Expect-Failure (New-Fixture "rollback-run") @() $result.Home
    Remove-Item Env:FORGE_TEST_FAIL_REPLACEMENT_AFTER -ErrorAction SilentlyContinue
    foreach ($name in @("forge.exe", "codex-threads.exe", "linear.exe", "mermaid.exe", "slack-agent.exe", "slack-query.exe")) {
        if ((Get-Content -LiteralPath (Join-Path $destination $name) -Raw).Trim() -ne "old-$name") { Fail "rollback left a mixed-version binary set" }
    }

    $binaryLinkReady = $false
    try {
        $linkTarget = Join-Path $result.Home "binary-target.exe"
        Set-Content -LiteralPath $linkTarget -Value "target"
        Remove-Item -LiteralPath (Join-Path $destination "codex-threads.exe") -Force
        New-Item -ItemType SymbolicLink -Path (Join-Path $destination "codex-threads.exe") -Target $linkTarget | Out-Null
        $binaryLinkReady = $true
    } catch {
        Write-Host "reparse-point binary fixture unavailable; native Windows installer contract remains covered by the POSIX fixture"
    }
    if ($binaryLinkReady) {
        [void](Expect-Failure (New-Fixture "binary-reparse") @() $result.Home)
    }

    $env:FORGE_TEST_GH_MODE = "success"
    $attest = Run-Installer (New-Fixture "attestation-success") @("-VerifyAttestation")
    $attestLog = Get-Content -LiteralPath $attest.GhLog -Raw
    if (-not $attest.Success -or $attestLog -notmatch "attestation verify") {
        Fail "explicit Windows attestation was not requested (success=$($attest.Success); error=$($attest.Error); log=<$attestLog>)"
    }
    $env:FORGE_TEST_GH_MODE = "fail"
    [void](Expect-Failure (New-Fixture "attestation-failure") @("-VerifyAttestation"))

    $source = Expect-Failure (New-Fixture "explicit-source") @("-BuildFromSource")
    if (-not ((Get-Content -LiteralPath $source.Log -Raw) -match "git clone")) { Fail "explicit Windows source mode did not invoke git" }

    $global:LASTEXITCODE = 0
    Write-Host "ok: PowerShell release installer adversarial fixtures passed"
} finally {
    $env:Path = $OriginalPath
    if ($null -eq $OriginalLocalAppData) { Remove-Item Env:LOCALAPPDATA -ErrorAction SilentlyContinue } else { $env:LOCALAPPDATA = $OriginalLocalAppData }
    Remove-Item -LiteralPath $TestRoot -Recurse -Force -ErrorAction SilentlyContinue
}
