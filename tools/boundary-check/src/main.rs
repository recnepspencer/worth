mod cargo_graph;
mod config;
mod certification_construction;
mod control_seal;
mod configured_dependency_denials;
mod configured_source_dependency_allowlists;
mod configured_source_identifier_denials;
mod configured_source_owner_isolations;
mod context_workspace_rules;
mod dependency_rules;
mod diagnostics;
mod facade_docs;
mod hook_authority;
mod legacy_references;
mod manifest_types;
mod naming;
mod one_build;
mod query_audience;
mod sealed_sources;
mod scan;
mod scan_profile;
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
    } = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let profile = scan_profile::Session::new(profile_path);
    let result = scan::run(root, config_path, snapshot_mode, width);
    drop(profile);
    match result {
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
            "--update-snapshots" => snapshot_mode = SnapshotMode::Update,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Arguments {
        root,
        config_path: config,
        format,
        snapshot_mode,
        profile_path: profile,
        width,
    })
}
