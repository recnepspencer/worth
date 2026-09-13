use crate::cargo_graph::{normalize_str, package_name_from_manifest, parse_workspace_manifest};
use crate::config::{Road1Config, SubworkspaceConfig};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use crate::manifest_types::WorkspaceManifest;
use std::path::Path;

pub(crate) fn validate_root_and_subworkspaces(
    root: &Path,
    config: &Road1Config,
) -> Result<Vec<Diagnostic>, String> {
    let root_manifest_path = root.join(&config.root_manifest);
    let root_manifest = parse_workspace_manifest(&root_manifest_path)?;

    let mut diagnostics = Vec::new();
    validate_root_manifest(
        &root_manifest,
        config,
        &root_manifest_path,
        &mut diagnostics,
    );
    validate_root_metadata_paths(&root_manifest, root, &mut diagnostics);
    validate_root_workspace_members(root, config, &mut diagnostics)?;

    for subworkspace in &config.subworkspaces {
        validate_subworkspace(root, subworkspace, &mut diagnostics)?;
    }

    Ok(diagnostics)
}

fn validate_root_manifest(
    manifest: &WorkspaceManifest,
    config: &Road1Config,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(workspace) = manifest.workspace.as_ref() else {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            path.display().to_string(),
            "root manifest is not a workspace manifest",
        ));
        return;
    };

    let Some(exclude) = workspace.exclude.as_ref() else {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            path.display().to_string(),
            "root workspace must declare exclude",
        ));
        return;
    };
    for forbidden_prefix in &config.forbidden_root_prefixes {
        let expected_exclusion = format!("{}/*", forbidden_prefix.trim_end_matches('/'));
        if !exclude.iter().any(|entry| entry == &expected_exclusion) {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc5002SubworkspaceContractViolation,
                path.display().to_string(),
                format!("root workspace must exclude {expected_exclusion}"),
            ));
        }
    }

    let worth_topology = workspace
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.worth_topology.as_ref());
    let Some(worth_topology) = worth_topology else {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            path.display().to_string(),
            "root workspace must declare workspace.metadata.worth_topology",
        ));
        return;
    };
    if worth_topology.role.as_deref() != Some("thin_orchestrator") {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            path.display().to_string(),
            "root worth_topology.role must be thin_orchestrator",
        ));
    }
    if worth_topology.forbidden_member_prefixes.as_ref() != Some(&config.forbidden_root_prefixes) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            path.display().to_string(),
            "root forbidden_member_prefixes must match boundary-check config",
        ));
    }
}

fn validate_root_metadata_paths(
    manifest: &WorkspaceManifest,
    root: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let worth_topology = manifest
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.metadata.as_ref())
        .and_then(|metadata| metadata.worth_topology.as_ref());
    let Some(worth_topology) = worth_topology else {
        return;
    };
    let Some(manifest_path) = worth_topology.boundary_check_manifest.as_ref() else {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            "root",
            "root worth_topology.boundary_check_manifest missing",
        ));
        return;
    };
    let Some(config_path) = worth_topology.boundary_check_config.as_ref() else {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            "root",
            "root worth_topology.boundary_check_config missing",
        ));
        return;
    };
    if !root.join(manifest_path).is_file() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            manifest_path,
            "declared boundary_check_manifest does not exist",
        ));
    }
    if !root.join(config_path).is_file() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            config_path,
            "declared boundary_check_config does not exist",
        ));
    }
}

fn validate_root_workspace_members(
    root: &Path,
    config: &Road1Config,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), String> {
    let manifest = parse_workspace_manifest(&root.join("Cargo.toml"))?;
    let members = manifest
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.members.as_ref())
        .cloned()
        .unwrap_or_default();
    for member in &members {
        for forbidden_prefix in &config.forbidden_root_prefixes {
            let normalized = normalize_str(forbidden_prefix);
            if member.contains(&normalized) {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::Bc5001RootOwnsRoad1Package,
                    member,
                    "root workspace illegally owns a Road 1 package",
                ));
            }
        }
    }
    Ok(())
}

fn validate_subworkspace(
    root: &Path,
    config: &SubworkspaceConfig,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), String> {
    let workspace_root = root.join(&config.path);
    let manifest_path = workspace_root.join("Cargo.toml");
    let readme_path = workspace_root.join("README.md");
    let crates_path = workspace_root.join("crates");

    if !manifest_path.is_file() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            manifest_path.display().to_string(),
            "missing subworkspace manifest",
        ));
        return Ok(());
    }
    if !readme_path.is_file() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            readme_path.display().to_string(),
            "missing subworkspace charter",
        ));
    }
    if !crates_path.is_dir() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            crates_path.display().to_string(),
            "missing crates lane",
        ));
        return Ok(());
    }

    let manifest = parse_workspace_manifest(&manifest_path)?;
    let workspace_members = manifest
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.members.as_deref())
        .unwrap_or_default();
    let worth_topology = manifest
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.metadata.as_ref())
        .and_then(|metadata| metadata.worth_topology.as_ref());
    let Some(worth_topology) = worth_topology else {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            manifest_path.display().to_string(),
            "subworkspace worth_topology metadata missing",
        ));
        return Ok(());
    };
    if worth_topology.role.as_deref() != Some("road1_subworkspace") {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            manifest_path.display().to_string(),
            "subworkspace role must be road1_subworkspace",
        ));
    }
    if worth_topology.member_lane.as_deref() != Some(&config.member_lane) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            manifest_path.display().to_string(),
            "member lane mismatch",
        ));
    }
    if worth_topology.allowed_crate_prefixes.as_ref() != Some(&config.allowed_crate_prefixes) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            manifest_path.display().to_string(),
            "allowed crate prefixes mismatch",
        ));
    }
    let expected_lane = workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if worth_topology.constitutional_lane.as_deref() != Some(expected_lane) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5002SubworkspaceContractViolation,
            manifest_path.display().to_string(),
            "constitutional lane must match folder name",
        ));
    }

    for entry in std::fs::read_dir(&crates_path)
        .map_err(|e| format!("read crates lane {}: {e}", crates_path.display()))?
    {
        let entry = entry.map_err(|e| format!("read crates lane entry: {e}"))?;
        let path = entry.path();
        if entry.file_name() == ".gitkeep" && path.is_file() {
            continue;
        }
        if !path.is_dir() {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc5002SubworkspaceContractViolation,
                path.display().to_string(),
                "unexpected non-directory in crates lane",
            ));
            continue;
        }
        let manifest_path = path.join("Cargo.toml");
        if !manifest_path.is_file() {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc5002SubworkspaceContractViolation,
                manifest_path.display().to_string(),
                "crate lane directory missing Cargo.toml",
            ));
            continue;
        }
        let crate_name = package_name_from_manifest(&manifest_path)?;
        let relative_member = format!("crates/{crate_name}");
        if !workspace_members
            .iter()
            .any(|member| workspace_member_matches(member, &relative_member))
        {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc5002SubworkspaceContractViolation,
                manifest_path.display().to_string(),
                "crate-lane package must be admitted by workspace.members",
            ));
        }
        if !config
            .allowed_crate_prefixes
            .iter()
            .any(|prefix| crate_name.starts_with(prefix))
        {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc5002SubworkspaceContractViolation,
                crate_name,
                "crate is outside the allowed prefixes for this subworkspace",
            ));
        }
    }

    Ok(())
}

fn workspace_member_matches(pattern: &str, candidate: &str) -> bool {
    let pattern = pattern.replace('\\', "/");
    pattern == candidate
        || pattern
            .strip_suffix('*')
            .is_some_and(|prefix| candidate.starts_with(prefix))
}
