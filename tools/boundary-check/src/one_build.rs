use crate::diagnostics::{Diagnostic, DiagnosticCode};
use proc_macro2::{TokenStream, TokenTree};
use std::{collections::BTreeSet, fs, path::Path};

const ROOTS: &[&str] = &[
    "crates/worth-relational",
    "crates/worth-execution",
    "crates/worth-signal",
    "crates/worth-signal-wasm",
    "crates/worth-runtime-bridge",
    "crates/worth-runtime-world",
    "workspaces/worth-query",
    "workspaces/worth-query-bank-world",
];

pub(crate) fn validate(root: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for owner in ROOTS {
        // Contract fixtures may supply a smaller repository: govern owners present.
        if matches!(root.join(owner).try_exists(), Ok(false)) {
            continue;
        }
        let files = match crate::governed_sources::files_under(&root.join(owner)) {
            Ok(files) => files,
            Err(error) => {
                diagnostics.push(failure(owner, error));
                continue;
            }
        };
        for file in files {
            let path = file
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            match fs::read_to_string(&file) {
                Ok(source) if file.file_name().is_some_and(|name| name == "Cargo.toml") => {
                    diagnostics.extend(manifest(&path, &source))
                }
                Ok(source) => diagnostics.extend(source_predicates(&path, &source)),
                Err(error) => diagnostics.push(failure(&path, error.to_string())),
            }
        }
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

fn manifest(path: &str, source: &str) -> Vec<Diagnostic> {
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
    inspect_manifest_tables(path, &value, &explicitly_named, &mut diagnostics);
    diagnostics
}

fn inspect_manifest_tables(
    path: &str,
    value: &toml::Value,
    explicit: &BTreeSet<&str>,
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
                inspect_manifest_tables(path, child, explicit, diagnostics);
            }
        }
        toml::Value::Array(values) => {
            for child in values {
                inspect_manifest_tables(path, child, explicit, diagnostics);
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
