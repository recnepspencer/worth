use std::path::Path;

use crate::cargo_graph::cargo_metadata;
use crate::config::{DependencyDenialConfig, DependencyTargetAllowlistConfig};
use crate::diagnostics::{Diagnostic, DiagnosticCode};

pub(crate) fn validate_configured_dependency_denials(
    root: &Path,
    rules: &[DependencyDenialConfig],
) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    for rule in rules {
        let manifest = root.join(&rule.workspace_manifest);
        let metadata = cargo_metadata(root, &manifest)?;
        for package in metadata.packages {
            diagnostics.extend(diagnostics_for_package(
                &package.name,
                &package.manifest_path,
                package
                    .dependencies
                    .iter()
                    .map(|dependency| dependency.name.as_str()),
                rule,
            ));
        }
    }
    Ok(diagnostics)
}

pub(crate) fn validate_dependency_target_allowlists(
    root: &Path,
    rules: &[DependencyTargetAllowlistConfig],
) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    for rule in rules {
        let metadata = cargo_metadata(root, &root.join(&rule.workspace_manifest))?;
        for package in metadata.packages {
            diagnostics.extend(diagnostics_for_target_allowlist_package(
                &package.name,
                &package.manifest_path,
                package
                    .dependencies
                    .iter()
                    .map(|dependency| dependency.name.as_str()),
                rule,
            ));
        }
    }
    Ok(diagnostics)
}

fn diagnostics_for_target_allowlist_package<'a>(
    package_name: &str,
    manifest_path: &str,
    dependencies: impl IntoIterator<Item = &'a str>,
    rule: &DependencyTargetAllowlistConfig,
) -> Vec<Diagnostic> {
    if !rule
        .governed_source_prefixes
        .iter()
        .any(|prefix| package_name.starts_with(prefix))
        || rule
            .allowed_sources
            .iter()
            .any(|source| source == package_name)
    {
        return Vec::new();
    }
    dependencies
        .into_iter()
        .filter(|dependency| *dependency == rule.target)
        .map(|dependency| {
            Diagnostic::new(
                DiagnosticCode::Bc2001BandDependencyViolation,
                manifest_path,
                format!(
                    "{package_name} must not depend on {dependency}: {}",
                    rule.guidance
                ),
            )
        })
        .collect()
}

fn diagnostics_for_package<'a>(
    package_name: &str,
    manifest_path: &str,
    dependencies: impl IntoIterator<Item = &'a str>,
    rule: &DependencyDenialConfig,
) -> Vec<Diagnostic> {
    if !rule.sources.iter().any(|source| source == package_name)
        && !rule
            .source_prefixes
            .iter()
            .any(|prefix| package_name.starts_with(prefix))
    {
        return Vec::new();
    }
    dependencies
        .into_iter()
        .filter(|dependency| {
            rule.forbidden_targets
                .iter()
                .any(|target| target == dependency)
                || rule
                    .forbidden_target_prefixes
                    .iter()
                    .any(|prefix| dependency.starts_with(prefix))
        })
        .map(|dependency| {
            Diagnostic::new(
                DiagnosticCode::Bc2001BandDependencyViolation,
                manifest_path,
                format!(
                    "{package_name} must not depend on {dependency}: {}",
                    rule.guidance
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{diagnostics_for_package, diagnostics_for_target_allowlist_package};
    use crate::config::{DependencyDenialConfig, DependencyTargetAllowlistConfig};

    #[test]
    fn configured_isolation_fence_rejects_store_and_recovery_but_accepts_lower_format() {
        let config: crate::config::Road1Config =
            toml::from_str(include_str!("../config/road1.toml")).unwrap();
        let rule = config
            .dependency_denials
            .iter()
            .find(|rule| {
                rule.sources
                    .iter()
                    .any(|source| source == "worth-store-physical-isolation")
                    && rule
                        .forbidden_targets
                        .iter()
                        .any(|target| target == "worth-store")
            })
            .expect("the checked-in configuration must fence isolation from Store");
        for forbidden in [
            "worth-store",
            "worth-store-recovery-physics",
            "worth-store-recovery-runtime",
        ] {
            let diagnostics = diagnostics_for_package(
                "worth-store-physical-isolation",
                "crates/worth-store-physical-isolation/Cargo.toml",
                ["worth-store-physical-format", forbidden],
                rule,
            );
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(
                diagnostics[0].code().as_str(),
                "BC2001_BAND_DEPENDENCY_VIOLATION"
            );
            assert!(diagnostics[0].message().contains(&format!(
                "worth-store-physical-isolation must not depend on {forbidden}:"
            )));
        }
    }

    #[test]
    fn adding_a_forbidden_lower_to_operations_edge_emits_the_exact_boundary_diagnostic() {
        let rule = DependencyDenialConfig {
            workspace_manifest: "workspaces/worth-store/Cargo.toml".into(),
            sources: vec!["worth-store-physical-backend".into()],
            source_prefixes: Vec::new(),
            forbidden_targets: vec!["worth-store-operations".into()],
            forbidden_target_prefixes: Vec::new(),
            guidance: "lower Store owners cannot depend on Operations".into(),
        };
        let diagnostics = diagnostics_for_package(
            "worth-store-physical-backend",
            "crates/worth-store-physical-backend/Cargo.toml",
            ["sha2", "worth-store-operations"],
            &rule,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code().as_str(),
            "BC2001_BAND_DEPENDENCY_VIOLATION"
        );
        assert!(diagnostics[0]
            .message()
            .contains("worth-store-physical-backend must not depend on worth-store-operations"));
    }

    #[test]
    fn target_allowlist_rejects_a_future_unlisted_store_signal_consumer() {
        let rule = DependencyTargetAllowlistConfig {
            workspace_manifest: "workspaces/worth-store/Cargo.toml".into(),
            governed_source_prefixes: vec!["worth-store".into()],
            target: "worth-signal".into(),
            allowed_sources: vec!["worth-store".into()],
            guidance: "Signal belongs to the composition owner".into(),
        };
        let diagnostics = diagnostics_for_target_allowlist_package(
            "worth-store-future",
            "crates/worth-store-future/Cargo.toml",
            ["worth-signal"],
            &rule,
        );
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("worth-store-future"));
        assert!(diagnostics_for_target_allowlist_package(
            "worth-store",
            "crates/worth-store/Cargo.toml",
            ["worth-signal"],
            &rule,
        )
        .is_empty());
    }

    #[test]
    fn source_prefix_denial_rejects_a_future_signal_store_dependency() {
        let rule = DependencyDenialConfig {
            workspace_manifest: "Cargo.toml".into(),
            sources: Vec::new(),
            source_prefixes: vec!["worth-signal".into()],
            forbidden_targets: Vec::new(),
            forbidden_target_prefixes: vec!["worth-store".into()],
            guidance: "generic Signal cannot depend on Store".into(),
        };
        let diagnostics = diagnostics_for_package(
            "worth-signal-future",
            "crates/worth-signal-future/Cargo.toml",
            ["worth-store-physical-backend"],
            &rule,
        );
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn runtime_bridge_denial_rejects_every_query_package_edge() {
        let rule = DependencyDenialConfig {
            workspace_manifest: "Cargo.toml".into(),
            sources: vec!["worth-runtime-bridge".into()],
            source_prefixes: Vec::new(),
            forbidden_targets: Vec::new(),
            forbidden_target_prefixes: vec!["worth-query".into()],
            guidance: "Bridge cannot import Query authority".into(),
        };
        for dependency in [
            "worth-query",
            "worth-query-declaration",
            "worth-query-installation",
            "worth-query-replay",
        ] {
            let diagnostics = diagnostics_for_package(
                "worth-runtime-bridge",
                "crates/worth-runtime-bridge/Cargo.toml",
                [dependency],
                &rule,
            );
            assert_eq!(diagnostics.len(), 1, "missing fence for {dependency}");
        }
    }
}
