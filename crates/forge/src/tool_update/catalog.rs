use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    Packages,
    Rustup,
    Uv,
    UvTools,
    CargoInstalls,
    Gum,
}

pub(super) struct TargetCatalogEntry {
    pub(super) target: Target,
    pub(super) id: &'static str,
    aliases: &'static [&'static str],
}

pub(super) const TARGET_CATALOG: &[TargetCatalogEntry] = &[
    TargetCatalogEntry {
        target: Target::Packages,
        id: "packages",
        aliases: &["package-manager", "system", "brew", "homebrew", "winget"],
    },
    TargetCatalogEntry {
        target: Target::Rustup,
        id: "rustup",
        aliases: &["rust", "rust-toolchain", "rust-toolchains"],
    },
    TargetCatalogEntry {
        target: Target::Uv,
        id: "uv",
        aliases: &["uv-self"],
    },
    TargetCatalogEntry {
        target: Target::UvTools,
        id: "uv-tools",
        aliases: &["uv-tool"],
    },
    TargetCatalogEntry {
        target: Target::CargoInstalls,
        id: "cargo-installs",
        aliases: &["cargo", "cargo-install"],
    },
    TargetCatalogEntry {
        target: Target::Gum,
        id: "gum",
        aliases: &[],
    },
];

impl Target {
    #[cfg(test)]
    pub(super) fn id(self) -> &'static str {
        TARGET_CATALOG
            .iter()
            .find(|entry| entry.target == self)
            .map(|entry| entry.id)
            .expect("target catalog missing target")
    }

    pub(super) fn from_raw(raw: &str) -> Result<Self> {
        for entry in TARGET_CATALOG {
            if entry.id == raw || entry.aliases.contains(&raw) {
                return Ok(entry.target);
            }
        }
        bail!("unknown global tool updater: {raw}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docs_cover_default_target_catalog() {
        let docs = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/forge.md"));
        for entry in TARGET_CATALOG {
            assert!(
                docs.contains(&format!("- `{}`", entry.id)),
                "docs/forge.md default target list is missing `{}`",
                entry.id
            );
            assert!(
                docs.contains(&format!("| `{}` |", entry.id)),
                "docs/forge.md source table is missing `{}`",
                entry.id
            );
        }
    }

    #[test]
    fn readme_covers_bootstrap_and_global_update_boundary() {
        let readme = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md"));
        for expected in [
            "Homebrew",
            "WinGet is Windows-only",
            "rustup",
            "uv-installed tools",
            "cargo-installed commands",
            "`gum`",
            "Project dependency updates are intentionally out of scope",
        ] {
            assert!(readme.contains(expected), "README.md missing `{expected}`");
        }
    }
}
