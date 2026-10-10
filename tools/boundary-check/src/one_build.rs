use crate::diagnostics::{Diagnostic, DiagnosticCode};
use proc_macro2::{TokenStream, TokenTree};
use std::{collections::BTreeSet, fs, path::Path};

// Every workspace/package root is governed unless its team is named here.
const OTHER_TEAMS: &[&str] = &[
    "workspaces/worth-store",
    "workspaces/worth-ui",
    "crates/worth-proof",
];
mod owned_dependencies;
use owned_dependencies::OwnedDependencies;

fn governed(path: &str) -> bool {
    !OTHER_TEAMS.iter().any(|owner| {
        path == *owner
            || path
                .strip_prefix(owner)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

pub(crate) fn validate(root: &Path) -> Vec<Diagnostic> {
    let files = match crate::governed_sources::files_under(root) {
        Ok(files) => files,
        Err(error) => return vec![failure(".", error)],
    };
    let mut diagnostics = Vec::new();
    let mut sources = Vec::new();
    let mut manifests = Vec::new();
    for file in files {
        let path = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if !governed(&path) {
            continue;
        }
        match fs::read_to_string(&file) {
            Ok(source) if file.file_name().is_some_and(|name| name == "Cargo.toml") => {
                manifests.push((path, source))
            }
            Ok(source) => sources.push((path, source)),
            Err(error) => diagnostics.push(failure(&path, error.to_string())),
        }
    }
    let owners = OwnedDependencies::from_manifests(&manifests);
    for (path, source) in manifests {
        diagnostics.extend(manifest_with_owners(&path, &source, &owners));
    }
    for (path, source) in sources {
        diagnostics.extend(source_predicates(&path, &source));
    }
    diagnostics
}

fn allowed(path: &str, name: &str) -> bool {
    let signal = path.starts_with("crates/worth-signal/");
    let wasm = path.starts_with("crates/worth-signal-wasm/");
    ((signal || wasm)
        && matches!(
            name,
            "default" | "profile-compact" | "profile-standard" | "profile-extended"
        ))
        || (signal && name == "test-peak-allocation")
}

fn failure(path: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc9001OneBuild, path, message)
}

#[cfg(test)]
fn manifest(path: &str, source: &str) -> Vec<Diagnostic> {
    manifest_with_owners(path, source, &OwnedDependencies::default())
}

fn manifest_with_owners(path: &str, source: &str, owners: &OwnedDependencies) -> Vec<Diagnostic> {
    let value: toml::Value = match toml::from_str(source) {
        Ok(value) => value,
        Err(error) => {
            return vec![failure(
                path,
                format!("cannot inspect one-build manifest: {error}"),
            )]
        }
    };
    let mut diagnostics = Vec::new();
    let features = value.get("features").and_then(toml::Value::as_table);
    if let Some(features) = features {
        if features.is_empty() {
            diagnostics.push(failure(path, "one build forbids an empty features table"));
        }
        for name in features.keys().filter(|name| !allowed(path, name)) {
            diagnostics.push(failure(
                path,
                format!("one build forbids declared feature `{name}`"),
            ));
        }
    }
    let explicitly_named: BTreeSet<_> = features
        .into_iter()
        .flat_map(|features| features.values())
        .filter_map(toml::Value::as_array)
        .flatten()
        .filter_map(toml::Value::as_str)
        .filter_map(|name| name.strip_prefix("dep:"))
        .collect();
    inspect_manifest_tables(path, &value, &explicitly_named, owners, &mut diagnostics);
    diagnostics
}

fn inspect_manifest_tables(
    path: &str,
    value: &toml::Value,
    explicit: &BTreeSet<&str>,
    owners: &OwnedDependencies,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match value {
        toml::Value::Table(table) => {
            for (key, child) in table {
                if matches!(
                    key.as_str(),
                    "dependencies" | "dev-dependencies" | "build-dependencies"
                ) {
                    if let Some(dependencies) = child.as_table() {
                        for (name, dependency) in dependencies {
                            if dependency.get("features").is_some()
                                && owners.owns(path, name, dependency)
                            {
                                diagnostics.push(failure(path, format!("one build forbids dependency features on governed package `{name}`")));
                            }
                            if dependency.get("optional").and_then(toml::Value::as_bool)
                                == Some(true)
                                && !explicit.contains(name.as_str())
                                && !allowed(path, name)
                            {
                                diagnostics.push(failure(path, format!("one build forbids implicit optional-dependency feature `{name}`")));
                            }
                        }
                    }
                }
                if key == "required-features" {
                    for name in child
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(toml::Value::as_str)
                        .filter(|name| !allowed(path, name))
                    {
                        diagnostics.push(failure(
                            path,
                            format!("one build forbids target required-feature `{name}`"),
                        ));
                    }
                }
                inspect_manifest_tables(path, child, explicit, owners, diagnostics);
            }
        }
        toml::Value::Array(values) => {
            for child in values {
                inspect_manifest_tables(path, child, explicit, owners, diagnostics);
            }
        }
        _ => {}
    }
}

fn source_predicates(path: &str, source: &str) -> Vec<Diagnostic> {
    if !source.contains("feature") || !source.contains("cfg") {
        return Vec::new();
    }
    let tokens: TokenStream = match source.parse() {
        Ok(tokens) => tokens,
        Err(error) => {
            return vec![failure(
                path,
                format!("cannot inspect one-build source: {error}"),
            )]
        }
    };
    let mut names = BTreeSet::new();
    find_cfg(tokens, &mut names);
    names
        .into_iter()
        .filter(|name| !allowed(path, name))
        .map(|name| {
            failure(
                path,
                format!("one build forbids feature predicate `{name}`"),
            )
        })
        .collect()
}

fn identifier_matches(token: &TokenTree, names: &[&str]) -> bool {
    let TokenTree::Ident(value) = token else {
        return false;
    };
    let spelling = value.to_string();
    names.contains(&spelling.strip_prefix("r#").unwrap_or(&spelling))
}

fn find_cfg(tokens: TokenStream, names: &mut BTreeSet<String>) {
    let tokens: Vec<_> = tokens.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if identifier_matches(token, &["cfg", "cfg_attr"]) {
            let next = match tokens.get(index + 1) {
                Some(TokenTree::Punct(mark)) if mark.as_char() == '!' => tokens.get(index + 2),
                next => next,
            };
            if let Some(TokenTree::Group(group)) = next {
                find_features(group.stream(), names);
            }
        }
        if let TokenTree::Group(group) = token {
            find_cfg(group.stream(), names);
        }
    }
}

fn find_features(tokens: TokenStream, names: &mut BTreeSet<String>) {
    let tokens: Vec<_> = tokens.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if identifier_matches(token, &["feature"])
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(mark)) if mark.as_char() == '=')
        {
            let name = tokens
                .get(index + 2)
                .and_then(|value| syn::parse_str::<syn::LitStr>(&value.to_string()).ok())
                .map(|value| value.value())
                .unwrap_or_else(|| "<nonliteral feature>".into());
            names.insert(name);
        }
        if let TokenTree::Group(group) = token {
            find_features(group.stream(), names);
        }
    }
}

#[cfg(test)]
mod tests;
