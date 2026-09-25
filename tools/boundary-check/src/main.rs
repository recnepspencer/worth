mod cargo_graph;
mod config;
mod configured_dependency_denials;
mod configured_source_dependency_allowlists;
mod configured_source_identifier_denials;
mod context_workspace_rules;
mod dependency_rules;
mod diagnostics;
mod hook_authority;
mod legacy_references;
mod manifest_types;
mod naming;
mod query_audience;
mod seed_contracts;
mod snapshots;
mod source_rules;
mod subworkspace_rules;

use crate::cargo_graph::discover_road1_packages;
use crate::config::Road1Config;
use crate::configured_dependency_denials::{
    validate_configured_dependency_denials, validate_dependency_target_allowlists,
};
use crate::configured_source_dependency_allowlists::validate_source_dependency_allowlists;
use crate::configured_source_identifier_denials::validate_source_identifier_denials;
use crate::context_workspace_rules::validate_context_workspaces;
use crate::dependency_rules::{validate_dependency_rules, validate_worth_ui_query_edge};
use crate::diagnostics::{render_human, render_json, Diagnostic};
use crate::hook_authority::validate_hook_authority;
use crate::legacy_references::validate_legacy_references;
use crate::naming::validate_package_names;
use crate::query_audience::{validate_query_audience_facades, validate_query_audience_rules};
use crate::seed_contracts::validate_seed_crate_contracts;
use crate::snapshots::{SnapshotMode, SnapshotSession};
use crate::source_rules::{
    enforce_raw_geometry_denials, validate_source_rules, validate_workspace_source_reachability,
};
use crate::subworkspace_rules::validate_root_and_subworkspaces;
use std::env;
use std::fs;
use std::path::PathBuf;

enum OutputFormat {
    Human,
    Json,
}

fn main() {
    let (root, config_path, format, snapshot_mode) = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    match run(root, config_path, snapshot_mode) {
        Ok(()) => println!("boundary-check: Road 1 Cargo topology is valid"),
        Err(diagnostics) => {
            let output = match format {
                OutputFormat::Human => Ok(render_human(&diagnostics)),
                OutputFormat::Json => render_json(&diagnostics),
            };
            eprintln!("{}", output.unwrap_or_else(|error| error));
            std::process::exit(1);
        }
    }
}

fn parse_args() -> Result<(PathBuf, PathBuf, OutputFormat, SnapshotMode), String> {
    let mut root = PathBuf::from(".");
    let mut config = PathBuf::from("tools/boundary-check/config/road1.toml");
    let mut format = OutputFormat::Human;
    let mut snapshot_mode = SnapshotMode::Check;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = PathBuf::from(args.next().ok_or("missing --root value")?),
            "--config" => config = PathBuf::from(args.next().ok_or("missing --config value")?),
            "--format" => {
                let value = args.next().ok_or("missing --format value")?;
                format = match value.as_str() {
                    "human" => OutputFormat::Human,
                    "json" => OutputFormat::Json,
                    other => return Err(format!("unknown --format value: {other}")),
                };
            }
            "--update-snapshots" => snapshot_mode = SnapshotMode::Update,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok((root, config, format, snapshot_mode))
}

fn run(
    root: PathBuf,
    config_path: PathBuf,
    snapshot_mode: SnapshotMode,
) -> Result<(), Vec<Diagnostic>> {
    let root = fs::canonicalize(root).map_err(|error| {
        vec![Diagnostic::new(
            crate::diagnostics::DiagnosticCode::Bc5002SubworkspaceContractViolation,
            ".",
            format!("canonicalize root failed: {error}"),
        )]
    })?;
    let resolved_config_path = resolve_config_path(&root, &config_path);
    let config_text = match fs::read_to_string(&resolved_config_path) {
        Ok(text) => text,
        Err(error) => {
            return Err(vec![Diagnostic::new(
                crate::diagnostics::DiagnosticCode::Bc5002SubworkspaceContractViolation,
                resolved_config_path.display().to_string(),
                format!("read config failed: {error}"),
            )])
        }
    };
    let config: Road1Config = toml::from_str(&config_text).map_err(|error| {
        vec![Diagnostic::new(
            crate::diagnostics::DiagnosticCode::Bc5002SubworkspaceContractViolation,
            resolved_config_path.display().to_string(),
            format!("parse config failed: {error}"),
        )]
    })?;

    let mut diagnostics = Vec::<Diagnostic>::new();
    diagnostics.extend(validate_hook_authority(&root));
    diagnostics.extend(validate_context_workspaces(
        &root,
        &config.context_workspaces,
        &config.rule_contracts,
    ));
    diagnostics.extend(validate_machine_authority(
        &root,
        &resolved_config_path,
        &config.machine_authority,
    ));
    diagnostics.extend(
        validate_root_and_subworkspaces(&root, &config).map_err(|error| {
            vec![Diagnostic::new(
                crate::diagnostics::DiagnosticCode::Bc5002SubworkspaceContractViolation,
                "root",
                error,
            )]
        })?,
    );
    diagnostics.extend(
        validate_dependency_target_allowlists(&root, &config.dependency_target_allowlists)
            .map_err(|error| {
                vec![Diagnostic::new(
                    crate::diagnostics::DiagnosticCode::Bc2001BandDependencyViolation,
                    "dependency-target-allowlists",
                    error,
                )]
            })?,
    );
    diagnostics.extend(
        validate_source_dependency_allowlists(&root, &config.source_dependency_allowlists)
            .map_err(|error| {
                vec![Diagnostic::new(
                    crate::diagnostics::DiagnosticCode::Bc2001BandDependencyViolation,
                    "source-dependency-allowlists",
                    error,
                )]
            })?,
    );
    diagnostics.extend(validate_source_identifier_denials(
        &root,
        &config.source_identifier_denials,
    ));
    diagnostics.extend(enforce_raw_geometry_denials(
        &root,
        &config.raw_geometry_denials,
    ));
    diagnostics.extend(
        validate_configured_dependency_denials(&root, &config.dependency_denials).map_err(
            |error| {
                vec![Diagnostic::new(
                    crate::diagnostics::DiagnosticCode::Bc2001BandDependencyViolation,
                    "configured-dependency-denials",
                    error,
                )]
            },
        )?,
    );

    let packages = discover_road1_packages(&root, &config.subworkspaces).map_err(|error| {
        vec![Diagnostic::new(
            crate::diagnostics::DiagnosticCode::Bc5002SubworkspaceContractViolation,
            "configured-subworkspaces",
            error,
        )]
    })?;
    diagnostics.extend(validate_package_names(&packages, &config.naming));
    diagnostics.extend(validate_dependency_rules(
        &packages,
        &config.rule_contracts,
        &config.law_substrates,
    ));
    if let Some(contract) = &config.rule_contracts.worth_ui_query_edge {
        diagnostics.extend(
            validate_worth_ui_query_edge(&root, contract).map_err(|error| {
                vec![Diagnostic::new(
                    crate::diagnostics::DiagnosticCode::Bc5002SubworkspaceContractViolation,
                    "worth-ui-query-edge",
                    error,
                )]
            })?,
        );
        diagnostics.extend(
            validate_workspace_source_reachability(&root, &contract.workspace).map_err(
                |error| {
                    vec![Diagnostic::new(
                        crate::diagnostics::DiagnosticCode::Bc7003SourceReachability,
                        contract.workspace.clone(),
                        error,
                    )]
                },
            )?,
        );
    }
    diagnostics.extend(validate_query_audience_rules(
        &packages,
        &config.rule_contracts.query_audience,
    ));
    diagnostics.extend(
        validate_query_audience_facades(&root, &config.rule_contracts.query_audience).map_err(
            |error| {
                vec![Diagnostic::new(
                    crate::diagnostics::DiagnosticCode::Bc3003QueryAudienceFacadeContract,
                    "query-audience-facades",
                    error,
                )]
            },
        )?,
    );

    diagnostics.extend(
        validate_seed_crate_contracts(
            &root,
            &config.born_crates,
            &config.seed_skeletons,
            &config.subworkspaces,
        )
        .map_err(|error| {
            vec![Diagnostic::new(
                crate::diagnostics::DiagnosticCode::Bc5003SeedContractViolation,
                "seed-contracts",
                error,
            )]
        })?,
    );
    let snapshot_session = SnapshotSession::prepare(
        &root,
        snapshot_mode,
        &packages,
        &config.rule_contracts.query_audience,
    );
    diagnostics.extend(snapshot_session.preparation_diagnostics().iter().cloned());
    if let Some(facade_authority) = snapshot_session.facade_vocabulary_authority() {
        diagnostics.extend(
            validate_source_rules(
                &root,
                &config.subworkspaces,
                &config.naming,
                &config.law_substrates,
                &config.rule_contracts.query_audience,
                &facade_authority,
            )
            .map_err(|error| {
                vec![Diagnostic::new(
                    crate::diagnostics::DiagnosticCode::Bc7001AuthoritySealing,
                    "source-rules",
                    error,
                )]
            })?,
        );
    }
    diagnostics.extend(
        validate_legacy_references(&root, &config.legacy_reference_ratchet).map_err(|error| {
            vec![Diagnostic::new(
                crate::diagnostics::DiagnosticCode::Bc6002LegacyReferenceBaseline,
                config.legacy_reference_ratchet.snapshot.clone(),
                error,
            )]
        })?,
    );

    let constitutional_laws_are_green = diagnostics.is_empty();
    let finalization = snapshot_session
        .finalize(&root, constitutional_laws_are_green)
        .map_err(|diagnostic| vec![diagnostic])?;
    diagnostics.extend(finalization.diagnostics);
    for path in finalization.updated_paths {
        println!("boundary-check: updated {}", path.display());
    }

    if diagnostics.is_empty() {
        return Ok(());
    }

    Err(diagnostics)
}

fn resolve_config_path(root: &std::path::Path, config_path: &PathBuf) -> PathBuf {
    if config_path.is_absolute() {
        return config_path.clone();
    }
    root.join(config_path)
}

fn validate_machine_authority(
    root: &std::path::Path,
    resolved_config_path: &std::path::Path,
    machine_authority: &crate::config::MachineAuthorityConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let canonical_path = root.join(&machine_authority.canonical_config);
    if canonical_path != resolved_config_path {
        diagnostics.push(Diagnostic::new(
            crate::diagnostics::DiagnosticCode::Bc5002SubworkspaceContractViolation,
            resolved_config_path.display().to_string(),
            format!(
                "loaded config path does not match machine_authority.canonical_config {}",
                machine_authority.canonical_config
            ),
        ));
    }
    for doc_path in &machine_authority.mirrored_docs {
        if !root.join(doc_path).is_file() {
            diagnostics.push(Diagnostic::with_legal_home(
                crate::diagnostics::DiagnosticCode::Bc5002SubworkspaceContractViolation,
                doc_path,
                "machine_authority mirrored doc is missing",
                format!(
                    "tools/boundary-check/config/road1.toml [machine_authority.mirrored_docs]; restore `{doc_path}` or remove the stale configured entry"
                ),
            ));
        }
    }
    diagnostics
}
