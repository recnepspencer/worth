//! Preserve configured rule and directory traversal order before file work.
use super::{unreadable_source_diagnostic, Diagnostic, SourceIdentifierDenialConfig};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) enum Input<'a> {
    Diagnostic(Diagnostic),
    File {
        path: PathBuf,
        rule: &'a SourceIdentifierDenialConfig,
    },
}

pub(super) fn collect<'a>(
    workspace: &Path,
    rules: &'a [SourceIdentifierDenialConfig],
) -> Vec<Input<'a>> {
    let mut inputs = Vec::new();
    for rule in rules {
        visit(workspace, &workspace.join(&rule.root), rule, &mut inputs);
    }
    inputs
}

fn excluded(workspace: &Path, path: &Path, rule: &SourceIdentifierDenialConfig) -> bool {
    rule.exclude_paths
        .iter()
        .map(|value| workspace.join(value))
        .any(|value| path == value || path.starts_with(value))
}

fn visit<'a>(
    workspace: &Path,
    path: &Path,
    rule: &'a SourceIdentifierDenialConfig,
    inputs: &mut Vec<Input<'a>>,
) {
    if excluded(workspace, path, rule) {
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => {
            inputs.push(Input::Diagnostic(unreadable_source_diagnostic(
                workspace, path, &error,
            )));
            return;
        }
    };
    for entry in entries {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(error) => {
                inputs.push(Input::Diagnostic(unreadable_source_diagnostic(
                    workspace, path, &error,
                )));
                continue;
            }
        };
        if excluded(workspace, &path, rule) {
            continue;
        }
        if path.is_dir() {
            visit(workspace, &path, rule, inputs);
        } else if path.extension().is_some_and(|value| value == "rs") {
            inputs.push(Input::File { path, rule });
        }
    }
}
