//! Law 1, reach: discover every exported test door throughout its owning crate.
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use proc_macro2::{TokenStream, TokenTree};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
mod macro_items;
mod public_items;
mod shipping_sources;
mod source_regions;
const SCOPES: &[&str] = &[
    "crates/worth-relational",
    "crates/worth-execution",
    "crates/worth-signal",
    "crates/worth-signal-wasm",
    "crates/worth-runtime-bridge",
    "crates/worth-runtime-world",
    "workspaces/worth-query",
    "workspaces/worth-query-bank-world",
];
const CALLERS: &str = include_str!("../config/control-callers.txt");

pub(crate) type Doors = BTreeMap<String, BTreeSet<String>>;
pub(crate) fn load(root: &Path) -> (Vec<Diagnostic>, Doors) {
    let mut diagnostics = Vec::new();
    let mut doors = Doors::new();
    for scope in SCOPES {
        let directory = root.join(scope);
        if !directory.is_dir() {
            continue;
        }
        let files = match shipping_sources::files_under(&directory) {
            Ok(files) => files,
            Err(error) => {
                diagnostics.push(failure(scope, error));
                continue;
            }
        };
        for (file, source) in files {
            let path = file
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let Some(owner) = owner(&path) else {
                continue;
            };
            if !source.contains("_for_test") && !source.contains("ForTest") {
                continue;
            }
            match public_items::test_names(&source) {
                Ok(names) => {
                    for name in names {
                        doors.entry(name).or_default().insert(owner.clone());
                    }
                }
                Err(error) => diagnostics.push(failure(
                    &path,
                    format!("cannot derive sealed test items: {error}"),
                )),
            }
        }
    }
    (diagnostics, doors)
}

fn owner(path: &str) -> Option<String> {
    for scope in SCOPES {
        if let Some(tail) = path.strip_prefix(&format!("{scope}/")) {
            if scope.starts_with("workspaces/") {
                return tail
                    .strip_prefix("crates/")
                    .and_then(|tail| tail.split('/').next())
                    .map(|name| format!("{scope}/crates/{name}/"));
            }
            return Some(format!("{scope}/"));
        }
    }
    None
}

fn entries(text: &str) -> impl Iterator<Item = (&str, &str)> {
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once(' '))
}
fn failure(path: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc7012ControlSeal, path, message)
}
fn permitted(path: &str, owners: &BTreeSet<String>) -> bool {
    owners.iter().any(|owner| path.starts_with(owner))
        || entries(CALLERS).any(|(owner, caller)| owners.contains(owner) && caller == path)
}
pub(crate) fn references(path: &str, source: &str, doors: &Doors) -> Vec<Diagnostic> {
    let relevant: BTreeSet<_> = doors
        .keys()
        .filter(|name| source.contains(name.as_str()) && !permitted(path, &doors[*name]))
        .cloned()
        .collect();
    if relevant.is_empty() {
        return Vec::new();
    }
    let control_source = owner(path).is_some_and(|owner| {
        public_items::test_names(source).is_ok_and(|names| {
            names.into_iter().any(|name| {
                doors
                    .get(&name)
                    .is_some_and(|owners| owners.contains(&owner))
            })
        })
    });
    let tokens: TokenStream = match if control_source {
        source_regions::ordinary_tokens(source)
    } else {
        source
            .parse()
            .map_err(|error: proc_macro2::LexError| error.to_string())
    } {
        Ok(tokens) => tokens,
        Err(error) => {
            return vec![failure(
                path,
                format!("cannot inspect sealed control references: {error}"),
            )]
        }
    };
    let mut names = BTreeSet::new();
    collect(tokens, &relevant, &mut names);
    names
        .into_iter()
        .filter(|name| !permitted(path, &doors[name]))
        .map(|name| {
            failure(
                path,
                format!("sealed control name `{name}` may not be referenced from `{path}`"),
            )
        })
        .collect()
}
fn collect(tokens: TokenStream, doors: &BTreeSet<String>, names: &mut BTreeSet<String>) {
    for token in tokens {
        match token {
            TokenTree::Ident(name) => {
                let name = name.to_string();
                let name = name.strip_prefix("r#").unwrap_or(&name);
                if doors.contains(name) {
                    names.insert(name.into());
                }
            }
            TokenTree::Group(group) => collect(group.stream(), doors, names),
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests;
