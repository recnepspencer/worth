use crate::config::{ContextWorkspaceConfig, RuleContracts};
use crate::dependency_rules::replay_surface_label;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn validate_context_workspaces(
    root: &Path,
    workspaces: &[ContextWorkspaceConfig],
    contracts: &RuleContracts,
) -> Vec<Diagnostic> {
    workspaces
        .iter()
        .flat_map(|workspace| validate_context_workspace(root, workspace, contracts))
        .collect()
}

fn validate_context_workspace(
    root: &Path,
    workspace: &ContextWorkspaceConfig,
    contracts: &RuleContracts,
) -> Vec<Diagnostic> {
    let workspace_root = root.join(&workspace.path);
    let members = match workspace_members(&workspace_root.join("Cargo.toml")) {
        Ok(members) => members,
        Err(error) => return vec![contract_diagnostic(&workspace.path, error)],
    };
    let discovered = match discover_members(&workspace_root, &members) {
        Ok(discovered) => discovered,
        Err(error) => return vec![contract_diagnostic(&workspace.path, error)],
    };
    let mut diagnostics = Vec::new();
    for (relative, manifest) in discovered {
        match manifest_contract(&manifest) {
            Ok((package, dependencies)) => {
                if package != workspace.package_prefix
                    && !package.starts_with(&format!("{}-", workspace.package_prefix))
                {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticCode::Bc1001IllegalCrateName,
                        relative.clone(),
                        format!(
                            "context workspace package {package} is outside {}",
                            workspace.package_prefix
                        ),
                    ));
                }
                diagnostics.extend(validate_dependencies(
                    workspace,
                    &package,
                    &relative,
                    &dependencies,
                    contracts,
                ));
            }
            Err(error) => diagnostics.push(contract_diagnostic(&relative, error)),
        }
    }
    diagnostics
}

fn discover_members(
    workspace_root: &Path,
    members: &[String],
) -> Result<Vec<(String, PathBuf)>, String> {
    let mut discovered = BTreeSet::new();
    for member in members {
        if let Some(parent) = member.strip_suffix("/*") {
            let parent_root = workspace_root.join(parent);
            let entries = fs::read_dir(&parent_root)
                .map_err(|error| format!("read {}: {error}", parent_root.display()))?;
            for entry in entries {
                let entry = entry.map_err(|error| {
                    format!("read context workspace member entry failed: {error}")
                })?;
                let relative = format!("{parent}/{}", entry.file_name().to_string_lossy());
                insert_member(workspace_root, &relative, &mut discovered)?;
            }
        } else {
            insert_member(workspace_root, member, &mut discovered)?;
        }
    }
    Ok(discovered.into_iter().collect())
}

fn insert_member(
    workspace_root: &Path,
    relative: &str,
    discovered: &mut BTreeSet<(String, PathBuf)>,
) -> Result<(), String> {
    let manifest = workspace_root.join(relative).join("Cargo.toml");
    if !manifest.is_file() {
        return Err(format!(
            "workspace member {relative} omits {}",
            manifest.display()
        ));
    }
    discovered.insert((relative.replace('\\', "/"), manifest));
    Ok(())
}

fn validate_dependencies(
    workspace: &ContextWorkspaceConfig,
    package: &str,
    relative: &str,
    dependencies: &BTreeSet<String>,
    contracts: &RuleContracts,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for dependency in dependencies {
        if dependency.starts_with("worthy-") {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc2002WorthToWorthyInversion,
                relative.to_owned(),
                format!("{package} must not depend on higher-tier package {dependency}"),
            ));
        }
        if let Some(label) = replay_surface_label(dependency, contracts) {
            if !workspace
                .certification_packages
                .iter()
                .any(|certification| certification == package)
            {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::Bc4001OrdinaryReplayImport,
                    relative.to_owned(),
                    format!("{package} must not depend on {label} package {dependency}"),
                ));
            }
        }
    }
    diagnostics
}

fn manifest_contract(path: &Path) -> Result<(String, BTreeSet<String>), String> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))?;
    let package = value
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("{} omits package.name", path.display()))?;
    let mut dependencies = BTreeSet::new();
    collect_dependency_tables(&value, &mut dependencies);
    if let Some(targets) = value.get("target").and_then(toml::Value::as_table) {
        for target in targets.values() {
            collect_dependency_tables(target, &mut dependencies);
        }
    }
    Ok((package, dependencies))
}

fn collect_dependency_tables(value: &toml::Value, dependencies: &mut BTreeSet<String>) {
    for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(table) = value.get(table_name).and_then(toml::Value::as_table) else {
            continue;
        };
        for (alias, specification) in table {
            let package = specification
                .as_table()
                .and_then(|details| details.get("package"))
                .and_then(toml::Value::as_str)
                .unwrap_or(alias);
            dependencies.insert(package.to_owned());
        }
    }
}

fn contract_diagnostic(path: &str, message: String) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc5002SubworkspaceContractViolation,
        path.to_owned(),
        message,
    )
}

fn workspace_members(path: &Path) -> Result<Vec<String>, String> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))?;
    value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .ok_or_else(|| format!("{} omits workspace.members", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{QueryAudienceContract, ReplaySurfaceConfig};

    fn replay_contracts() -> RuleContracts {
        RuleContracts {
            query_audience: QueryAudienceContract {
                workspace: ".".to_owned(),
                engine_package: "worth-query".to_owned(),
                certification_package: None,
                certification_authority_packages: Vec::new(),
                certification_consumers: Vec::new(),
                internal_packages: Vec::new(),
                facade_surfaces: Vec::new(),
                audiences: Vec::new(),
            },
            replay_surfaces: vec![ReplaySurfaceConfig {
                label: "certification replay".to_owned(),
                package_prefixes: vec!["worth-query-replay".to_owned()],
                cert_domains: vec!["replay".to_owned()],
            }],
            band_rules: Vec::new(),
            worth_ui_query_edge: None,
        }
    }

    fn workspace() -> ContextWorkspaceConfig {
        ContextWorkspaceConfig {
            path: "workspaces/worth-ui".to_owned(),
            package_prefix: "worth-ui".to_owned(),
            certification_packages: vec!["worth-ui-certification".to_owned()],
        }
    }

    #[test]
    fn dependency_tables_include_aliases_and_target_specific_dependencies() {
        let value: toml::Value = toml::from_str(
            r#"
                [dependencies]
                renamed = { package = "worthy-alias", version = "1" }

                [target.'cfg(windows)'.dev-dependencies]
                worth-query-replay = "1"
            "#,
        )
        .unwrap();
        let mut dependencies = BTreeSet::new();
        collect_dependency_tables(&value, &mut dependencies);
        for target in value["target"].as_table().unwrap().values() {
            collect_dependency_tables(target, &mut dependencies);
        }
        assert_eq!(
            dependencies,
            BTreeSet::from(["worth-query-replay".to_owned(), "worthy-alias".to_owned()])
        );
    }

    #[test]
    fn ordinary_context_packages_reject_worthy_and_replay_dependencies() {
        let diagnostics = validate_dependencies(
            &workspace(),
            "worth-ui-runtime",
            "crates/worth-ui-runtime",
            &BTreeSet::from(["worthy-scene".to_owned(), "worth-query-replay".to_owned()]),
            &replay_contracts(),
        );
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::Bc2002WorthToWorthyInversion));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code() == DiagnosticCode::Bc4001OrdinaryReplayImport));
    }

    #[test]
    fn configured_certification_package_admits_replay_but_not_worthy() {
        let diagnostics = validate_dependencies(
            &workspace(),
            "worth-ui-certification",
            "crates/worth-ui-certification",
            &BTreeSet::from(["worthy-scene".to_owned(), "worth-query-replay".to_owned()]),
            &replay_contracts(),
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code(),
            DiagnosticCode::Bc2002WorthToWorthyInversion
        );
    }

    #[test]
    fn configured_workspace_discovery_includes_the_application_member() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let worth_ui = root.join("workspaces/worth-ui");
        let members = workspace_members(&worth_ui.join("Cargo.toml")).unwrap();
        let discovered = discover_members(&worth_ui, &members).unwrap();
        assert!(discovered
            .iter()
            .any(|(relative, _)| relative == "apps/platform-pulse"));
    }
}
