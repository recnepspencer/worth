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

pub(crate) fn discover_context_crates(
    root: &Path,
    workspace: &crate::authority_inputs::ContextWorkspaceSpec,
) -> Result<Vec<DiscoveredCrate>, String> {
    let workspace_root = root.join(&workspace.path);
    let members = workspace_members(&workspace_root.join("Cargo.toml"))?;
    let mut discovered = Vec::new();
    for relative_member in members {
        let crate_root = workspace_root.join(&relative_member);
        let manifest = crate_root.join("Cargo.toml");
        if !manifest.is_file() {
            return Err(format!(
                "workspace member {relative_member} omits {}",
                manifest.display()
            ));
        }
        let package = package_name_from_manifest(&manifest)?;
        if package != workspace.package_prefix
            && !package.starts_with(&format!("{}-", workspace.package_prefix))
        {
            return Err(format!(
                "{package} is outside context workspace prefix {}",
                workspace.package_prefix
            ));
        }
        discovered.push(DiscoveredCrate {
            package,
            relative_path: crate_root
                .strip_prefix(root)
                .map_err(|error| format!("strip root prefix: {error}"))?
                .to_string_lossy()
                .replace('\\', "/"),
        });
    }
    discovered.sort_by(|left, right| left.package.cmp(&right.package));
    Ok(discovered)
}

fn workspace_members(path: &Path) -> Result<Vec<String>, String> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))?;
    let members = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| format!("{} omits workspace.members", path.display()))?;
    let mut expanded = Vec::new();
    for member in members.iter().filter_map(toml::Value::as_str) {
        if let Some(parent) = member.strip_suffix("/*") {
            let parent_root = path.parent().unwrap_or_else(|| Path::new("")).join(parent);
            for entry in fs::read_dir(&parent_root)
                .map_err(|error| format!("read {}: {error}", parent_root.display()))?
            {
                let entry = entry.map_err(|error| format!("read workspace member: {error}"))?;
                expanded.push(format!("{parent}/{}", entry.file_name().to_string_lossy()));
            }
        } else {
            expanded.push(member.to_owned());
        }
    }
    expanded.sort();
    Ok(expanded)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_inputs::ContextWorkspaceSpec;

    #[test]
    fn context_discovery_includes_declared_application_members() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let workspace = ContextWorkspaceSpec {
            path: "workspaces/worth-ui".to_owned(),
            package_prefix: "worth-ui".to_owned(),
            certification_packages: vec!["worth-ui-certification".to_owned()],
        };
        let discovered = discover_context_crates(&root, &workspace).unwrap();
        assert!(discovered.iter().any(|member| {
            member.package == "worth-ui-platform-pulse"
                && member.relative_path == "workspaces/worth-ui/apps/platform-pulse"
        }));
    }
}
