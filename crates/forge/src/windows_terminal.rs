use std::{
    env, fs,
    fs::OpenOptions,
    io::Write as _,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};
use jsonc_parser::{
    ParseOptions,
    cst::{CstInputValue, CstObject, CstRootNode},
    json,
};
use serde::Serialize;

pub const GIT_BASH_GUID: &str = "{2ece5bfe-50ed-5f3a-ab87-5cd4baafed2b}";
pub const DEFAULT_THEME: &str = "dark";
pub const DEFAULT_FONT_FACE: &str = "Cascadia Mono";
pub const DEFAULT_GIT_BASH_COMMANDLINE: &str = r#""%PROGRAMFILES%\Git\bin\bash.exe" -li"#;

#[derive(Debug, Clone)]
pub struct Preferences {
    pub theme: String,
    pub font_face: String,
    pub git_bash_commandline: String,
}

#[derive(Debug, Serialize)]
pub struct ReconcileResult {
    pub settings_path: String,
    pub compliant: bool,
    pub changes: Vec<PreferenceChange>,
    #[serde(skip)]
    pub original: String,
    #[serde(skip)]
    pub rendered: String,
}

#[derive(Debug, Serialize)]
pub struct PreferenceChange {
    pub path: String,
    pub current: Option<String>,
    pub desired: String,
}

pub fn resolve_settings_path(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        if !path.is_absolute() {
            bail!(
                "Windows Terminal settings path must be absolute: {}",
                path.display()
            );
        }
        return Ok(path.to_path_buf());
    }

    if !cfg!(windows) {
        bail!("Windows Terminal preference commands require Windows or --settings <absolute-path>");
    }

    let local_app_data = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("LOCALAPPDATA is not set; use --settings <absolute-path>"))?;
    let candidates = [
        local_app_data
            .join("Packages")
            .join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
            .join("LocalState")
            .join("settings.json"),
        local_app_data
            .join("Packages")
            .join("Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe")
            .join("LocalState")
            .join("settings.json"),
        local_app_data
            .join("Microsoft")
            .join("Windows Terminal")
            .join("settings.json"),
    ];
    let existing = candidates
        .into_iter()
        .filter(|candidate| candidate.is_file())
        .collect::<Vec<_>>();
    match existing.as_slice() {
        [path] => Ok(path.clone()),
        [] => bail!(
            "Windows Terminal settings.json was not found; open Terminal once or use --settings <absolute-path>"
        ),
        _ => bail!(
            "multiple Windows Terminal settings files were found; select one with --settings <absolute-path>"
        ),
    }
}

pub fn reconcile(path: &Path, preferences: &Preferences) -> Result<ReconcileResult> {
    let existing = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read Windows Terminal settings {}",
            path.display()
        )
    })?;
    let root =
        CstRootNode::parse(&existing, &windows_terminal_parse_options()).map_err(|error| {
            anyhow!(
                "failed to parse Windows Terminal settings {}: {error}",
                path.display()
            )
        })?;
    let object = root.object_value().ok_or_else(|| {
        anyhow!(
            "Windows Terminal settings root must be an object: {}",
            path.display()
        )
    })?;
    let mut changes = Vec::new();

    ensure_string(&object, "theme", &preferences.theme, "theme", &mut changes)?;
    ensure_string(
        &object,
        "defaultProfile",
        GIT_BASH_GUID,
        "defaultProfile",
        &mut changes,
    )?;

    let profiles = object
        .object_value_or_create("profiles")
        .ok_or_else(|| anyhow!("Windows Terminal setting profiles must be an object"))?;
    let defaults = profiles
        .object_value_or_create("defaults")
        .ok_or_else(|| anyhow!("Windows Terminal setting profiles.defaults must be an object"))?;
    let font = defaults.object_value_or_create("font").ok_or_else(|| {
        anyhow!("Windows Terminal setting profiles.defaults.font must be an object")
    })?;
    ensure_string(
        &font,
        "face",
        &preferences.font_face,
        "profiles.defaults.font.face",
        &mut changes,
    )?;

    let list = profiles
        .array_value_or_create("list")
        .ok_or_else(|| anyhow!("Windows Terminal setting profiles.list must be an array"))?;
    let matches = list
        .elements()
        .into_iter()
        .filter_map(|node| node.as_object())
        .filter(profile_matches_git_bash)
        .collect::<Vec<_>>();
    let profile = match matches.as_slice() {
        [] => {
            changes.push(PreferenceChange {
                path: "profiles.list[git-bash]".to_string(),
                current: None,
                desired: "present".to_string(),
            });
            list.append(json!({
                guid: GIT_BASH_GUID,
                name: "Git Bash",
                commandline: (preferences.git_bash_commandline.clone()),
                hidden: false,
            }))
            .as_object()
            .expect("inserted Git Bash profile should be an object")
        }
        [profile] => profile.clone(),
        _ => bail!(
            "multiple Git Bash profiles match GUID {GIT_BASH_GUID} or name; remove the duplicate before applying preferences"
        ),
    };
    ensure_string(
        &profile,
        "guid",
        GIT_BASH_GUID,
        "profiles.list[git-bash].guid",
        &mut changes,
    )?;
    ensure_string(
        &profile,
        "name",
        "Git Bash",
        "profiles.list[git-bash].name",
        &mut changes,
    )?;
    ensure_string(
        &profile,
        "commandline",
        &preferences.git_bash_commandline,
        "profiles.list[git-bash].commandline",
        &mut changes,
    )?;
    ensure_bool(
        &profile,
        "hidden",
        false,
        "profiles.list[git-bash].hidden",
        &mut changes,
    )?;

    let rendered = root.to_string();
    Ok(ReconcileResult {
        settings_path: path.display().to_string(),
        compliant: changes.is_empty(),
        changes,
        original: existing,
        rendered,
    })
}

pub fn apply(path: &Path, expected: &str, rendered: &str) -> Result<()> {
    let current = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to re-read Windows Terminal settings {} before apply",
            path.display()
        )
    })?;
    if current != expected {
        bail!(
            "Windows Terminal settings changed while preferences were being applied; retry to avoid overwriting concurrent edits"
        );
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("settings path has no parent: {}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("settings path has no UTF-8 file name: {}", path.display()))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?
        .as_nanos();
    let temp_path = parent.join(format!(
        ".{file_name}.forge.{}.{nonce}.tmp",
        std::process::id()
    ));
    let mut temp_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .with_context(|| {
            format!(
                "failed to create temporary settings {}",
                temp_path.display()
            )
        })?;
    if let Err(error) = temp_file.write_all(rendered.as_bytes()) {
        let _ = fs::remove_file(&temp_path);
        return Err(error).with_context(|| {
            format!("failed to write temporary settings {}", temp_path.display())
        });
    }
    drop(temp_file);
    let permissions = fs::metadata(path)
        .with_context(|| format!("failed to inspect settings permissions {}", path.display()))?
        .permissions();
    if let Err(error) = fs::set_permissions(&temp_path, permissions) {
        let _ = fs::remove_file(&temp_path);
        return Err(error).with_context(|| {
            format!(
                "failed to preserve settings permissions on {}",
                temp_path.display()
            )
        });
    }
    if let Err(error) = replace_file(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error).with_context(|| {
            format!(
                "failed to replace Windows Terminal settings {}",
                path.display()
            )
        });
    }
    Ok(())
}

fn windows_terminal_parse_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    }
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    // SAFETY: both paths are NUL-terminated UTF-16 buffers that remain alive for the call.
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn profile_matches_git_bash(profile: &CstObject) -> bool {
    object_string(profile, "guid").is_some_and(|value| value.eq_ignore_ascii_case(GIT_BASH_GUID))
        || object_string(profile, "name")
            .is_some_and(|value| value.eq_ignore_ascii_case("Git Bash"))
}

fn object_string(object: &CstObject, name: &str) -> Option<String> {
    object
        .get(name)?
        .value()?
        .as_string_lit()?
        .decoded_value()
        .ok()
}

fn object_bool(object: &CstObject, name: &str) -> Option<bool> {
    Some(object.get(name)?.value()?.as_boolean_lit()?.value())
}

fn ensure_string(
    object: &CstObject,
    name: &str,
    desired: &str,
    path: &str,
    changes: &mut Vec<PreferenceChange>,
) -> Result<()> {
    let current = object_string(object, name);
    if current.as_deref() == Some(desired) {
        return Ok(());
    }
    changes.push(PreferenceChange {
        path: path.to_string(),
        current,
        desired: desired.to_string(),
    });
    match object.get(name) {
        Some(property) => property.set_value(CstInputValue::from(desired)),
        None => {
            object.append(name, CstInputValue::from(desired));
        }
    }
    Ok(())
}

fn ensure_bool(
    object: &CstObject,
    name: &str,
    desired: bool,
    path: &str,
    changes: &mut Vec<PreferenceChange>,
) -> Result<()> {
    let current = object_bool(object, name);
    if current == Some(desired) {
        return Ok(());
    }
    changes.push(PreferenceChange {
        path: path.to_string(),
        current: current.map(|value| value.to_string()),
        desired: desired.to_string(),
    });
    match object.get(name) {
        Some(property) => property.set_value(CstInputValue::from(desired)),
        None => {
            object.append(name, CstInputValue::from(desired));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        env::temp_dir().join(format!("forge-{name}-{}-{nonce}.json", std::process::id()))
    }

    fn preferences() -> Preferences {
        Preferences {
            theme: "dark".to_string(),
            font_face: "CaskaydiaMono Nerd Font".to_string(),
            git_bash_commandline: DEFAULT_GIT_BASH_COMMANDLINE.to_string(),
        }
    }

    #[test]
    fn reconcile_preserves_comments_and_unrelated_settings() {
        let path = test_path("windows-terminal-preserve");
        fs::write(
            &path,
            r#"{
    // keep this comment
    "copyOnSelect": true,
    "profiles": {
        "defaults": {},
        "list": [
            { "guid": "{powershell}", "name": "PowerShell" },
        ],
    },
}
"#,
        )
        .expect("write fixture");

        let result = reconcile(&path, &preferences()).expect("reconcile settings");

        assert!(!result.compliant);
        assert!(result.rendered.contains("// keep this comment"));
        assert!(result.rendered.contains("\"copyOnSelect\": true"));
        assert!(result.rendered.contains("\"name\": \"PowerShell\""));
        assert!(result.rendered.contains(GIT_BASH_GUID));
        assert!(result.rendered.contains("CaskaydiaMono Nerd Font"));
        fs::remove_file(path).expect("remove fixture");
    }

    #[test]
    fn reconcile_updates_existing_git_bash_profile_without_duplication() {
        let path = test_path("windows-terminal-update");
        fs::write(
            &path,
            r#"{
  "theme": "light",
  "defaultProfile": "PowerShell",
  "profiles": {
    "defaults": { "font": { "face": "Consolas" } },
    "list": [
      { "name": "Git Bash", "hidden": true, "commandline": "old" }
    ]
  }
}
"#,
        )
        .expect("write fixture");

        let result = reconcile(&path, &preferences()).expect("reconcile settings");

        assert_eq!(result.rendered.matches(GIT_BASH_GUID).count(), 2);
        assert_eq!(result.rendered.matches("\"name\": \"Git Bash\"").count(), 1);
        assert!(result.rendered.contains("\"hidden\": false"));
        fs::remove_file(path).expect("remove fixture");
    }

    #[test]
    fn reconcile_is_idempotent_and_apply_replaces_file() {
        let path = test_path("windows-terminal-apply");
        fs::write(&path, "{}\n").expect("write fixture");

        let first = reconcile(&path, &preferences()).expect("first reconcile");
        apply(&path, &first.original, &first.rendered).expect("apply settings");
        let second = reconcile(&path, &preferences()).expect("second reconcile");

        assert!(second.compliant);
        assert!(second.changes.is_empty());
        fs::remove_file(path).expect("remove fixture");
    }

    #[test]
    fn reconcile_rejects_duplicate_git_bash_profiles() {
        let path = test_path("windows-terminal-duplicate");
        fs::write(
            &path,
            format!(
                r#"{{
  "profiles": {{
    "list": [
      {{ "guid": "{GIT_BASH_GUID}", "name": "Git Bash" }},
      {{ "name": "git bash" }}
    ]
  }}
}}
"#
            ),
        )
        .expect("write fixture");

        let error = reconcile(&path, &preferences()).expect_err("duplicate should fail");

        assert!(error.to_string().contains("multiple Git Bash profiles"));
        fs::remove_file(path).expect("remove fixture");
    }

    #[test]
    fn apply_rejects_concurrent_settings_changes() {
        let path = test_path("windows-terminal-concurrent");
        fs::write(&path, "{}\n").expect("write fixture");
        let result = reconcile(&path, &preferences()).expect("reconcile settings");
        fs::write(&path, "{ \"concurrent\": true }\n").expect("write concurrent edit");

        let error = apply(&path, &result.original, &result.rendered)
            .expect_err("concurrent edit should fail");

        assert!(error.to_string().contains("changed while preferences"));
        assert_eq!(
            fs::read_to_string(&path).expect("read concurrent edit"),
            "{ \"concurrent\": true }\n"
        );
        fs::remove_file(path).expect("remove fixture");
    }

    #[test]
    fn explicit_settings_path_must_be_absolute() {
        let error = resolve_settings_path(Some(Path::new("settings.json")))
            .expect_err("relative path should fail");

        assert!(error.to_string().contains("must be absolute"));
    }

    #[test]
    fn reconcile_rejects_jsonc_extensions_windows_terminal_does_not_support() {
        let path = test_path("windows-terminal-loose-jsonc");
        fs::write(&path, "{ theme: 'dark' }\n").expect("write fixture");

        let error = reconcile(&path, &preferences()).expect_err("loose JSONC should fail");

        assert!(error.to_string().contains("failed to parse"));
        fs::remove_file(path).expect("remove fixture");
    }
}
