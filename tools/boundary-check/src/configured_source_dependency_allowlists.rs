use std::collections::BTreeSet;
use std::path::Path;

use crate::cargo_graph::cargo_metadata;
use crate::config::{SourceDependencyAllowlistConfig, SourceDependencyContractConfig};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::manifest_types::CargoMetadataDependency;

pub(crate) fn validate_source_dependency_allowlists(
    root: &Path,
    rules: &[SourceDependencyAllowlistConfig],
) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    for rule in rules {
        let metadata = cargo_metadata(root, &root.join(&rule.workspace_manifest))?;
        for package in metadata.packages {
            diagnostics.extend(diagnostics_for_source_allowlist_package(
                &package.name,
                &package.manifest_path,
                package.dependencies.iter(),
                rule,
            ));
        }
    }
    Ok(diagnostics)
}

fn diagnostics_for_source_allowlist_package<'a>(
    package_name: &str,
    manifest_path: &str,
    dependencies: impl IntoIterator<Item = &'a CargoMetadataDependency>,
    rule: &SourceDependencyAllowlistConfig,
) -> Vec<Diagnostic> {
    if !rule.sources.iter().any(|source| source == package_name) {
        return Vec::new();
    }
    let dependencies = dependencies.into_iter().collect::<Vec<_>>();
    let context = SourcePackageContext {
        name: package_name,
        manifest_path,
        rule,
    };
    let mut diagnostics = unlisted_target_diagnostics(&context, &dependencies);
    diagnostics.extend(dependency_contract_diagnostics(&context, &dependencies));
    diagnostics
}

struct SourcePackageContext<'a> {
    name: &'a str,
    manifest_path: &'a str,
    rule: &'a SourceDependencyAllowlistConfig,
}

fn unlisted_target_diagnostics(
    context: &SourcePackageContext<'_>,
    dependencies: &[&CargoMetadataDependency],
) -> Vec<Diagnostic> {
    dependencies
        .iter()
        .filter(|dependency| {
            !context
                .rule
                .allowed_targets
                .iter()
                .any(|allowed| allowed == &dependency.name)
        })
        .map(|dependency| {
            Diagnostic::new(
                DiagnosticCode::Bc2001BandDependencyViolation,
                context.manifest_path,
                format!(
                    "{package_name} must not depend on unlisted target {}: {}",
                    dependency.name,
                    context.rule.guidance,
                    package_name = context.name,
                ),
            )
        })
        .collect()
}

fn dependency_contract_diagnostics(
    context: &SourcePackageContext<'_>,
    dependencies: &[&CargoMetadataDependency],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for contract in &context.rule.dependency_contracts {
        let Some(dependency) = dependencies
            .iter()
            .find(|dependency| dependency.name == contract.target)
        else {
            diagnostics.push(source_dependency_contract_diagnostic(
                context,
                &contract.target,
                "required dependency is missing",
            ));
            continue;
        };
        if let Some(detail) = dependency_contract_mismatch(dependency, contract) {
            diagnostics.push(source_dependency_contract_diagnostic(
                context,
                &contract.target,
                &detail,
            ));
        }
    }
    diagnostics
}

fn dependency_contract_mismatch(
    dependency: &CargoMetadataDependency,
    contract: &SourceDependencyContractConfig,
) -> Option<String> {
    let actual_features = dependency
        .features
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected_features = contract
        .features
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    (dependency.req != contract.version_requirement
        || dependency.uses_default_features != contract.uses_default_features
        || actual_features != expected_features)
        .then(|| {
            format!(
                "expected version `{}`, default_features={}, features={expected_features:?}; found version `{}`, default_features={}, features={actual_features:?}",
                contract.version_requirement,
                contract.uses_default_features,
                dependency.req,
                dependency.uses_default_features,
            )
        })
}

fn source_dependency_contract_diagnostic(
    context: &SourcePackageContext<'_>,
    dependency: &str,
    detail: &str,
) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc2001BandDependencyViolation,
        context.manifest_path,
        format!(
            "{package_name} dependency contract for {dependency} is invalid: {detail}: {}",
            context.rule.guidance,
            package_name = context.name,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::diagnostics_for_source_allowlist_package;
    use crate::config::{SourceDependencyAllowlistConfig, SourceDependencyContractConfig};
    use crate::manifest_types::CargoMetadataDependency;

    #[test]
    fn source_allowlist_rejects_unlisted_pulse_binding_owner_dependency() {
        let rule = pulse_source_allowlist();
        let diagnostics = diagnostics_for_source_allowlist_package(
            "worth-ui-platform-pulse",
            "apps/platform-pulse/Cargo.toml",
            [
                dependency("worth-ui"),
                dependency("worth-ui-query-binding"),
                pulse_uiautomation_dependency(),
                pulse_winsafe_dependency(),
                pulse_xcap_dependency(),
            ]
            .iter(),
            &rule,
        );
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("worth-ui-query-binding"));
    }

    #[test]
    fn source_allowlist_accepts_only_the_frozen_pulse_composition_edges() {
        let rule = pulse_source_allowlist();
        let diagnostics = diagnostics_for_source_allowlist_package(
            "worth-ui-platform-pulse",
            "apps/platform-pulse/Cargo.toml",
            [
                dependency("worth-ui"),
                dependency("worth-ui-native-platform"),
                dependency("worth-query-decl"),
                dependency("worth-query-host"),
                dependency("notify"),
                dependency("serde"),
                dependency("serde_json"),
                dependency("win32job"),
                pulse_uiautomation_dependency(),
                pulse_winsafe_dependency(),
                pulse_xcap_dependency(),
            ]
            .iter(),
            &rule,
        );
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn source_allowlist_rejects_pulse_automation_feature_drift() {
        let rule = pulse_source_allowlist();
        let mut automation = pulse_uiautomation_dependency();
        automation.uses_default_features = true;
        automation.features.push("tree".into());
        let diagnostics = diagnostics_for_source_allowlist_package(
            "worth-ui-platform-pulse",
            "apps/platform-pulse/Cargo.toml",
            [
                dependency("worth-ui"),
                dependency("worth-ui-native-platform"),
                automation,
                pulse_winsafe_dependency(),
                pulse_xcap_dependency(),
            ]
            .iter(),
            &rule,
        );
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("dependency contract"));
    }

    fn pulse_source_allowlist() -> SourceDependencyAllowlistConfig {
        SourceDependencyAllowlistConfig {
            workspace_manifest: "workspaces/worth-ui/Cargo.toml".into(),
            sources: vec!["worth-ui-platform-pulse".into()],
            allowed_targets: vec![
                "worth-ui".into(),
                "worth-ui-native-platform".into(),
                "worth-query-decl".into(),
                "worth-query-host".into(),
                "notify".into(),
                "serde".into(),
                "serde_json".into(),
                "uiautomation".into(),
                "win32job".into(),
                "winsafe".into(),
                "xcap".into(),
            ],
            dependency_contracts: vec![
                SourceDependencyContractConfig {
                    target: "uiautomation".into(),
                    version_requirement: "=0.25.0".into(),
                    uses_default_features: false,
                    features: vec!["control".into(), "input".into()],
                },
                SourceDependencyContractConfig {
                    target: "winsafe".into(),
                    version_requirement: "=0.0.28".into(),
                    uses_default_features: true,
                    features: vec!["user".into()],
                },
                SourceDependencyContractConfig {
                    target: "xcap".into(),
                    version_requirement: "=0.9.7".into(),
                    uses_default_features: false,
                    features: Vec::new(),
                },
            ],
            guidance:
                "the pulse is a downstream composition root with observation-only serialization"
                    .into(),
        }
    }

    fn dependency(name: &str) -> CargoMetadataDependency {
        CargoMetadataDependency {
            kind: None,
            name: name.into(),
            req: "*".into(),
            features: Vec::new(),
            uses_default_features: true,
        }
    }

    fn pulse_uiautomation_dependency() -> CargoMetadataDependency {
        CargoMetadataDependency {
            kind: None,
            name: "uiautomation".into(),
            req: "=0.25.0".into(),
            features: vec!["control".into(), "input".into()],
            uses_default_features: false,
        }
    }

    fn pulse_winsafe_dependency() -> CargoMetadataDependency {
        CargoMetadataDependency {
            kind: None,
            name: "winsafe".into(),
            req: "=0.0.28".into(),
            features: vec!["user".into()],
            uses_default_features: true,
        }
    }

    fn pulse_xcap_dependency() -> CargoMetadataDependency {
        CargoMetadataDependency {
            kind: None,
            name: "xcap".into(),
            req: "=0.9.7".into(),
            features: Vec::new(),
            uses_default_features: false,
        }
    }
}
