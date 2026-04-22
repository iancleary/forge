use std::{
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use cli_core::{
    ErrorBody, OutputMode, emit_output, ensure_owner_only_permissions, format_error_human,
    prepare_config_dir, print_error_json, resolve_config_dir,
};
use serde::Serialize;

const CONFIG_DIR_ENV: &str = "FORGE_MERMAID_CONFIG_DIR";
const CONFIG_SUBDIR: &str = "mermaid";
const TOOL_DIR_NAME: &str = "tool";
const TOOL_PACKAGE_NAME: &str = "forge-mermaid-tool";
const MERMAID_CLI_PACKAGE: &str = "@mermaid-js/mermaid-cli";
const DEFAULT_PACKAGE_VERSION: &str = "11.12.0";
const LOCAL_BINARY_NAME: &str = if cfg!(windows) { "mmdc.cmd" } else { "mmdc" };

#[derive(Parser, Debug)]
#[command(name = "mermaid")]
#[command(about = "Render Mermaid diagrams through a stable Forge wrapper")]
#[command(
    after_help = "Output:\n  - Default output is human-readable.\n  - Use --json for compact machine-readable JSON.\n  - Errors follow the same rule: human-readable by default, compact JSON with --json."
)]
struct Cli {
    #[arg(long, global = true, help = "Emit compact machine-readable JSON")]
    json: bool,
    #[arg(
        long,
        global = true,
        help = "Override the Mermaid tool root; defaults to ~/.config/forge/mermaid/tool"
    )]
    tool_root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Doctor,
    #[command(subcommand)]
    Tool(ToolCommand),
    Render(RenderArgs),
}

#[derive(Subcommand, Debug)]
enum ToolCommand {
    Install(ToolInstallArgs),
}

#[derive(Args, Debug)]
struct ToolInstallArgs {
    #[arg(
        long,
        default_value = DEFAULT_PACKAGE_VERSION,
        help = "Pinned @mermaid-js/mermaid-cli package version"
    )]
    package_version: String,
}

#[derive(Args, Debug)]
struct RenderArgs {
    #[arg(long, help = "Input Mermaid file path or '-' for stdin")]
    input: String,
    #[arg(long, help = "Output SVG/PNG/PDF/Markdown path")]
    output: PathBuf,
    #[arg(long)]
    config_file: Option<PathBuf>,
    #[arg(long)]
    css_file: Option<PathBuf>,
    #[arg(long)]
    puppeteer_config_file: Option<PathBuf>,
    #[arg(long)]
    background: Option<String>,
    #[arg(long)]
    theme: Option<String>,
    #[arg(long)]
    width: Option<u32>,
    #[arg(long)]
    height: Option<u32>,
    #[arg(long)]
    scale: Option<f32>,
    #[arg(long)]
    pdf_fit: bool,
    #[arg(long)]
    quiet: bool,
    #[arg(long, value_enum, default_value_t = ToolMode::Auto)]
    tool_mode: ToolMode,
    #[arg(
        long,
        default_value = DEFAULT_PACKAGE_VERSION,
        help = "Pinned @mermaid-js/mermaid-cli package version when dlx is used"
    )]
    package_version: String,
}

#[derive(Copy, Clone, Debug, Serialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ToolMode {
    Auto,
    Installed,
    Dlx,
}

#[derive(Debug, Serialize)]
struct DoctorResult {
    config_dir: String,
    tool_root: String,
    default_package: String,
    default_package_version: String,
    node: CommandCheck,
    pnpm: CommandCheck,
    installed_binary: ToolBinaryStatus,
    recommended_install_command: String,
}

#[derive(Debug, Serialize)]
struct CommandCheck {
    binary: String,
    available: bool,
    version: Option<String>,
}

#[derive(Debug, Serialize)]
struct ToolBinaryStatus {
    path: String,
    exists: bool,
}

#[derive(Debug, Serialize)]
struct ToolInstallResult {
    tool_root: String,
    manifest_path: String,
    installed_binary_path: String,
    package_spec: String,
}

#[derive(Debug, Serialize)]
struct RenderResult {
    input: String,
    output: String,
    tool_mode: ToolMode,
    tool_root: String,
    package_spec: String,
    command: Vec<String>,
}

#[derive(Debug)]
struct Runner {
    mode: ToolMode,
    command: Vec<String>,
}

fn main() {
    let args = env::args_os().collect::<Vec<_>>();
    let wants_json = args.iter().any(|arg| arg.to_str() == Some("--json"));
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            let exit_code = err.exit_code();
            match err.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    let _ = err.print();
                }
                _ if wants_json => {
                    print_error_json(&ErrorBody {
                        code: "invalid_usage".to_string(),
                        message: err.to_string(),
                    });
                }
                _ => {
                    eprintln!("{err}");
                }
            }
            std::process::exit(exit_code);
        }
    };

    if let Err(err) = run(cli) {
        let body = ErrorBody {
            code: "runtime_error".to_string(),
            message: err.to_string(),
        };
        if wants_json {
            print_error_json(&body);
        } else {
            eprintln!("{}", format_error_human("mermaid", &body));
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let output = OutputMode::from_json_flag(cli.json);
    let config_dir = resolve_mermaid_config_dir()?;
    let tool_root = resolve_tool_root(cli.tool_root.as_deref())?;

    match cli.command {
        Command::Doctor => {
            let data = doctor(&config_dir, &tool_root);
            emit_output(output, data, format_doctor_human)
        }
        Command::Tool(ToolCommand::Install(args)) => {
            let data = install_tool(&config_dir, &tool_root, &args.package_version)?;
            emit_output(output, data, format_tool_install_human)
        }
        Command::Render(args) => {
            let data = render(&tool_root, args)?;
            emit_output(output, data, format_render_human)
        }
    }
}

fn doctor(config_dir: &Path, tool_root: &Path) -> DoctorResult {
    let installed_binary = installed_binary_path(tool_root);
    DoctorResult {
        config_dir: config_dir.display().to_string(),
        tool_root: tool_root.display().to_string(),
        default_package: MERMAID_CLI_PACKAGE.to_string(),
        default_package_version: DEFAULT_PACKAGE_VERSION.to_string(),
        node: command_check("node", &["--version"]),
        pnpm: command_check("pnpm", &["--version"]),
        installed_binary: ToolBinaryStatus {
            path: installed_binary.display().to_string(),
            exists: installed_binary.exists(),
        },
        recommended_install_command: "mermaid tool install".to_string(),
    }
}

fn install_tool(
    config_dir: &Path,
    tool_root: &Path,
    package_version: &str,
) -> Result<ToolInstallResult> {
    let package_spec = package_spec(package_version);
    prepare_config_dir(config_dir)?;
    fs::create_dir_all(tool_root)
        .with_context(|| format!("failed to create {}", tool_root.display()))?;
    ensure_owner_only_permissions(tool_root, true)?;

    let manifest_path = write_manifest_if_missing(tool_root)?;
    let args = vec![
        "--dir".to_string(),
        tool_root.display().to_string(),
        "add".to_string(),
        "--save-exact".to_string(),
        "--save-dev".to_string(),
        package_spec.clone(),
    ];
    run_command("pnpm", &args)?;

    Ok(ToolInstallResult {
        tool_root: tool_root.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        installed_binary_path: installed_binary_path(tool_root).display().to_string(),
        package_spec,
    })
}

fn render(tool_root: &Path, args: RenderArgs) -> Result<RenderResult> {
    ensure_parent_dir(&args.output)?;

    let runner = resolve_runner(tool_root, args.tool_mode, &args.package_version)?;
    let mut command = runner.command.clone();
    command.extend(build_render_args(&args));
    run_command(&command[0], &command[1..])?;

    Ok(RenderResult {
        input: args.input,
        output: args.output.display().to_string(),
        tool_mode: runner.mode,
        tool_root: tool_root.display().to_string(),
        package_spec: package_spec(&args.package_version),
        command,
    })
}

fn resolve_mermaid_config_dir() -> Result<PathBuf> {
    resolve_config_dir(CONFIG_DIR_ENV, CONFIG_SUBDIR, true)
}

fn resolve_tool_root(tool_root: Option<&Path>) -> Result<PathBuf> {
    Ok(match tool_root {
        Some(path) => path.to_path_buf(),
        None => resolve_mermaid_config_dir()?.join(TOOL_DIR_NAME),
    })
}

fn installed_binary_path(tool_root: &Path) -> PathBuf {
    tool_root
        .join("node_modules")
        .join(".bin")
        .join(LOCAL_BINARY_NAME)
}

fn write_manifest_if_missing(tool_root: &Path) -> Result<PathBuf> {
    let manifest_path = tool_root.join("package.json");
    if manifest_path.exists() {
        return Ok(manifest_path);
    }

    let manifest = format!(
        concat!(
            "{{\n",
            "  \"name\": \"{name}\",\n",
            "  \"private\": true,\n",
            "  \"type\": \"module\"\n",
            "}}\n"
        ),
        name = TOOL_PACKAGE_NAME
    );
    fs::write(&manifest_path, manifest)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;
    Ok(manifest_path)
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("output path must have a parent directory"))?;
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    Ok(())
}

fn resolve_runner(tool_root: &Path, requested: ToolMode, package_version: &str) -> Result<Runner> {
    let installed_binary = installed_binary_path(tool_root);
    let installed = installed_binary.exists();
    let mode = match requested {
        ToolMode::Auto if installed => ToolMode::Installed,
        ToolMode::Auto => ToolMode::Dlx,
        ToolMode::Installed if !installed => {
            bail!(
                "installed tool mode requested but no local mmdc binary exists at {}",
                installed_binary.display()
            );
        }
        other => other,
    };

    let command = match mode {
        ToolMode::Installed => vec![
            "pnpm".to_string(),
            "--dir".to_string(),
            tool_root.display().to_string(),
            "exec".to_string(),
            "mmdc".to_string(),
        ],
        ToolMode::Dlx => vec![
            "pnpm".to_string(),
            "dlx".to_string(),
            package_spec(package_version),
            "mmdc".to_string(),
        ],
        ToolMode::Auto => unreachable!("auto mode should be resolved before command construction"),
    };

    Ok(Runner { mode, command })
}

fn build_render_args(args: &RenderArgs) -> Vec<String> {
    let mut out = vec![
        "--input".to_string(),
        args.input.clone(),
        "--output".to_string(),
        args.output.display().to_string(),
    ];

    if let Some(path) = &args.config_file {
        out.push("--configFile".to_string());
        out.push(path.display().to_string());
    }
    if let Some(path) = &args.css_file {
        out.push("--cssFile".to_string());
        out.push(path.display().to_string());
    }
    if let Some(path) = &args.puppeteer_config_file {
        out.push("--puppeteerConfigFile".to_string());
        out.push(path.display().to_string());
    }
    if let Some(background) = &args.background {
        out.push("--backgroundColor".to_string());
        out.push(background.clone());
    }
    if let Some(theme) = &args.theme {
        out.push("--theme".to_string());
        out.push(theme.clone());
    }
    if let Some(width) = args.width {
        out.push("--width".to_string());
        out.push(width.to_string());
    }
    if let Some(height) = args.height {
        out.push("--height".to_string());
        out.push(height.to_string());
    }
    if let Some(scale) = args.scale {
        out.push("--scale".to_string());
        out.push(scale.to_string());
    }
    if args.pdf_fit {
        out.push("--pdfFit".to_string());
    }
    if args.quiet {
        out.push("--quiet".to_string());
    }

    out
}

fn command_check(binary: &str, args: &[&str]) -> CommandCheck {
    match ProcessCommand::new(binary).args(args).output() {
        Ok(output) if output.status.success() => {
            let version = first_non_empty_line(&output.stdout, &output.stderr);
            CommandCheck {
                binary: binary.to_string(),
                available: true,
                version: if version.is_empty() {
                    None
                } else {
                    Some(version)
                },
            }
        }
        Ok(_) | Err(_) => CommandCheck {
            binary: binary.to_string(),
            available: false,
            version: None,
        },
    }
}

fn first_non_empty_line(stdout: &[u8], stderr: &[u8]) -> String {
    for candidate in [stdout, stderr] {
        if let Some(line) = String::from_utf8_lossy(candidate)
            .lines()
            .find(|line| !line.trim().is_empty())
        {
            return line.trim().to_string();
        }
    }
    String::new()
}

fn run_command(program: &str, args: &[String]) -> Result<()> {
    let output = ProcessCommand::new(program)
        .args(args)
        .output()
        .with_context(|| format!("failed to start {}", render_command(program, args)))?;

    if output.status.success() {
        return Ok(());
    }

    let mut message = String::new();
    let _ = writeln!(
        message,
        "{} exited with status {}",
        render_command(program, args),
        output
            .status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_string())
    );

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        let _ = writeln!(message, "{stderr}");
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        let _ = writeln!(message, "{stdout}");
    }

    bail!(message.trim_end().to_string())
}

fn render_command(program: &str, args: &[String]) -> String {
    let mut parts = vec![program.to_string()];
    parts.extend(args.iter().cloned());
    parts.join(" ")
}

fn package_spec(version: &str) -> String {
    format!("{MERMAID_CLI_PACKAGE}@{version}")
}

fn format_doctor_human(result: &DoctorResult) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "config dir: {}", result.config_dir);
    let _ = writeln!(out, "tool root: {}", result.tool_root);
    let _ = writeln!(
        out,
        "node: {}{}",
        if result.node.available {
            "ok"
        } else {
            "missing"
        },
        result
            .node
            .version
            .as_ref()
            .map(|v| format!(" ({v})"))
            .unwrap_or_default()
    );
    let _ = writeln!(
        out,
        "pnpm: {}{}",
        if result.pnpm.available {
            "ok"
        } else {
            "missing"
        },
        result
            .pnpm
            .version
            .as_ref()
            .map(|v| format!(" ({v})"))
            .unwrap_or_default()
    );
    let _ = writeln!(
        out,
        "installed mmdc: {} ({})",
        if result.installed_binary.exists {
            "present"
        } else {
            "missing"
        },
        result.installed_binary.path
    );
    let _ = writeln!(
        out,
        "package: {}@{}",
        result.default_package, result.default_package_version
    );
    let _ = writeln!(out, "next: {}", result.recommended_install_command);
    out.trim_end().to_string()
}

fn format_tool_install_human(result: &ToolInstallResult) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "installed {}", result.package_spec);
    let _ = writeln!(out, "tool root: {}", result.tool_root);
    let _ = writeln!(out, "manifest: {}", result.manifest_path);
    let _ = writeln!(out, "binary: {}", result.installed_binary_path);
    out.trim_end().to_string()
}

fn format_render_human(result: &RenderResult) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "rendered {}", result.output);
    let _ = writeln!(out, "input: {}", result.input);
    let _ = writeln!(out, "mode: {}", tool_mode_name(result.tool_mode));
    let _ = writeln!(out, "command: {}", result.command.join(" "));
    out.trim_end().to_string()
}

fn tool_mode_name(mode: ToolMode) -> &'static str {
    match mode {
        ToolMode::Auto => "auto",
        ToolMode::Installed => "installed",
        ToolMode::Dlx => "dlx",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_spec_is_pinned() {
        assert_eq!(package_spec("11.12.0"), "@mermaid-js/mermaid-cli@11.12.0");
    }

    #[test]
    fn build_render_args_maps_flags() {
        let args = RenderArgs {
            input: "diagram.mmd".to_string(),
            output: PathBuf::from("out/diagram.svg"),
            config_file: Some(PathBuf::from("mermaid.config.json")),
            css_file: Some(PathBuf::from("diagram.css")),
            puppeteer_config_file: Some(PathBuf::from("puppeteer.json")),
            background: Some("transparent".to_string()),
            theme: Some("dark".to_string()),
            width: Some(1600),
            height: Some(900),
            scale: Some(2.0),
            pdf_fit: true,
            quiet: true,
            tool_mode: ToolMode::Auto,
            package_version: DEFAULT_PACKAGE_VERSION.to_string(),
        };

        assert_eq!(
            build_render_args(&args),
            vec![
                "--input",
                "diagram.mmd",
                "--output",
                "out/diagram.svg",
                "--configFile",
                "mermaid.config.json",
                "--cssFile",
                "diagram.css",
                "--puppeteerConfigFile",
                "puppeteer.json",
                "--backgroundColor",
                "transparent",
                "--theme",
                "dark",
                "--width",
                "1600",
                "--height",
                "900",
                "--scale",
                "2",
                "--pdfFit",
                "--quiet",
            ]
        );
    }
}
