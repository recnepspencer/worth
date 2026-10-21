mod cargo_graph;
mod certification_construction;
mod config;
mod configured_dependency_denials;
mod configured_source_dependency_allowlists;
mod configured_source_identifier_denials;
mod configured_source_owner_isolations;
mod consumer_docs;
mod context_workspace_rules;
mod control_seal;
mod dependency_rules;
mod derived_skills;
mod diagnostics;
mod facade_docs;
mod governed_sources;
mod hook_authority;
mod legacy_references;
mod manifest_types;
mod naming;
mod one_build;
mod query_audience;
mod scan;
mod scan_profile;
mod sealed_sources;
mod seed_contracts;
mod snapshots;
mod source_documents;
mod source_rules;
mod source_syntax;
mod source_workers;
mod subworkspace_rules;

use crate::diagnostics::{render_human, render_json};
use crate::snapshots::SnapshotMode;
use std::{env, path::PathBuf};

struct Arguments {
    root: PathBuf,
    config_path: PathBuf,
    format: OutputFormat,
    snapshot_mode: SnapshotMode,
    profile_path: Option<PathBuf>,
    width: usize,
    write_guidance: bool,
    sealed_sources_only: bool,
}

enum OutputFormat {
    Human,
    Json,
}

fn main() {
    let Arguments {
        root,
        config_path,
        format,
        snapshot_mode,
        profile_path,
        width,
        write_guidance,
        sealed_sources_only,
    } = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    if write_guidance {
        match consumer_docs::write_guidance(&root) {
            Ok(()) => println!("boundary-check: generated guidance regions and derived skills"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        return;
    }
    let profile = scan_profile::Session::new(profile_path);
    let result = if sealed_sources_only {
        let diagnostics = sealed_sources::validate(&root);
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    } else {
        scan::run(root, config_path, snapshot_mode, width)
    };
    drop(profile);
    match result {
        Ok(()) if sealed_sources_only => println!("boundary-check: sealed source reach is valid"),
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

fn parse_args() -> Result<Arguments, String> {
    let mut root = PathBuf::from(".");
    let mut config = PathBuf::from("tools/boundary-check/config/road1.toml");
    let mut format = OutputFormat::Human;
    let mut snapshot_mode = SnapshotMode::Check;
    let mut profile = None;
    let mut width = std::thread::available_parallelism().map_or(1, usize::from);
    let mut write_guidance = false;
    let mut sealed_sources_only = false;
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
            "--profile" => {
                profile = Some(PathBuf::from(args.next().ok_or("missing --profile value")?))
            }
            "--width" => {
                width = args
                    .next()
                    .ok_or("missing --width value")?
                    .parse()
                    .map_err(|_| "--width must be a positive integer")?;
                if width == 0 {
                    return Err("--width must be a positive integer".into());
                }
            }
            "--write-guidance" => write_guidance = true,
            "--sealed-sources-only" => sealed_sources_only = true,
            "--update-snapshots" => snapshot_mode = SnapshotMode::Update,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    if write_guidance && matches!(snapshot_mode, SnapshotMode::Update) {
        return Err("guidance writes and snapshot updates are separate operations".into());
    }
    if sealed_sources_only && (write_guidance || matches!(snapshot_mode, SnapshotMode::Update)) {
        return Err("sealed-source validation cannot write guidance or snapshots".into());
    }
    Ok(Arguments {
        root,
        config_path: config,
        format,
        snapshot_mode,
        profile_path: profile,
        width,
        write_guidance,
        sealed_sources_only,
    })
}
