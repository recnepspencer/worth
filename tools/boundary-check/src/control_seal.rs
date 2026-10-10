//! Law 1, reach: derive doors from control files, never maintain a door-name list.
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use proc_macro2::{TokenStream, TokenTree};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
mod public_items;
const FILES: &str = include_str!("../config/control-files.txt");
const CALLERS: &str = include_str!("../config/control-callers.txt");

pub(crate) type Doors = BTreeMap<String, BTreeSet<String>>;
pub(crate) fn load(root: &Path) -> (Vec<Diagnostic>, Doors) {
    let mut diagnostics = Vec::new();
    let mut doors = Doors::new();
    for (owner, path) in entries(FILES) {
        let file = root.join(path);
        // Small tool contract repositories need not contain every platform owner.
        if !root.join(owner).exists() {
            continue;
        }
        let mut sources = vec![file.clone()];
        let children = file.with_extension("");
        if children.is_dir() {
            match crate::governed_sources::files_under(&children) {
                Ok(files) => sources.extend(
                    files
                        .into_iter()
                        .filter(|file| file.extension().is_some_and(|ext| ext == "rs")),
                ),
                Err(error) => diagnostics.push(failure(path, error)),
            }
        }
        for source_file in sources {
            match fs::read_to_string(&source_file)
                .and_then(|source| public_items::names(&source).map_err(std::io::Error::other))
            {
                Ok(names) => {
                    for name in names {
                        doors.entry(name).or_default().insert(owner.into());
                    }
                }
                Err(error) => diagnostics.push(failure(
                    path,
                    format!("cannot derive sealed control items: {error}"),
                )),
            }
        }
    }
    (diagnostics, doors)
}

fn entries(text: &str) -> impl Iterator<Item = (&str, &str)> {
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once(' '))
}
fn failure(path: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc7009ControlSeal, path, message)
}
fn permitted(path: &str, owners: &BTreeSet<String>) -> bool {
    owners.iter().any(|owner| path.starts_with(owner))
        || entries(CALLERS).any(|(owner, caller)| owners.contains(owner) && caller == path)
        // A sealed control may delegate to another owner's sealed control.
        || entries(FILES).any(|(_, control)| control == path)
}
pub(crate) fn references(path: &str, source: &str, doors: &Doors) -> Vec<Diagnostic> {
    let relevant: BTreeSet<_> = doors
        .keys()
        .filter(|name| source.contains(name.as_str()))
        .cloned()
        .collect();
    if relevant.is_empty() {
        return Vec::new();
    }
    let tokens: TokenStream = match source.parse() {
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
