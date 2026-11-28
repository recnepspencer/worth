use std::fs;
use std::path::Path;

pub(crate) struct DiscoveredCrate {
    pub(crate) package: String,
    pub(crate) relative_path: String,
}

pub(crate) struct GovernedCrateIdentity {
    pub(crate) tier: String,
    pub(crate) band: String,
    pub(crate) domain: String,
}

pub(crate) fn discover_born_crates(
    root: &Path,
    subworkspace_paths: &[String],
) -> Result<Vec<DiscoveredCrate>, String> {
    let mut discovered = Vec::new();
    for subworkspace in subworkspace_paths {
        let crates_root = root.join(subworkspace).join("crates");
        if !crates_root.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&crates_root)
            .map_err(|e| format!("read {}: {e}", crates_root.display()))?
        {
            let entry = entry.map_err(|e| format!("read {} entry: {e}", crates_root.display()))?;
            let crate_root = entry.path();
            let manifest_path = crate_root.join("Cargo.toml");
            if !crate_root.is_dir() || !manifest_path.is_file() {
                continue;
            }
            let relative_path = crate_root
                .strip_prefix(root)
                .map_err(|e| format!("strip root prefix from {}: {e}", crate_root.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            discovered.push(DiscoveredCrate {
                package: package_name_from_manifest(&manifest_path)?,
                relative_path,
            });
        }
    }
    discovered.sort_by(|left, right| left.package.cmp(&right.package));
    Ok(discovered)
}

pub(crate) fn parse_governed_crate_identity(
    package: &str,
) -> Result<GovernedCrateIdentity, String> {
    let parts = package.split('-').collect::<Vec<_>>();
    if parts.len() < 3 {
        return Err(format!(
            "{package} does not parse as {{tier}}-{{band}}-{{domain}}"
        ));
    }
    Ok(GovernedCrateIdentity {
        tier: parts[0].to_owned(),
        band: parts[1].to_owned(),
        domain: parts[2..].join("-"),
    })
}

fn package_name_from_manifest(path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    value
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(|name| name.as_str())
        .map(str::to_owned)
        .ok_or_else(|| format!("{} missing package.name", path.display()))
}
