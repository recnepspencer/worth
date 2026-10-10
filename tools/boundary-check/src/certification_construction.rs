use crate::diagnostics::{Diagnostic, DiagnosticCode};
use proc_macro2::{TokenStream, TokenTree};
use std::collections::BTreeSet;

const BRIDGE: &[&str] = &[
    "registered_authoritative_source",
    "with_authoritative_source",
    "from_owner_publication",
    "from_owner_partition_publication",
];
const LOOKUP: &str = "certification_query";
const CALLERS: &str = include_str!("../config/certification-construction-callers.txt");

fn failure(path: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc7010CertificationConstructionSeal,
        path,
        message,
    )
}

fn permitted(path: &str, name: &str) -> bool {
    let (class, owner) = if name == LOOKUP {
        (
            "installation",
            "workspaces/worth-query/crates/worth-query-installation/",
        )
    } else {
        ("bridge", "crates/worth-runtime-bridge/")
    };
    path.starts_with(owner)
        || CALLERS
            .lines()
            .filter_map(|line| line.split_once(' '))
            .any(|(allowed_class, allowed_path)| allowed_class == class && allowed_path == path)
}

pub(crate) fn source_references(path: &str, source: &str) -> Vec<Diagnostic> {
    if !BRIDGE
        .iter()
        .chain([&LOOKUP])
        .any(|name| source.contains(*name))
    {
        return Vec::new();
    }
    let tokens: TokenStream = match source.parse() {
        Ok(tokens) => tokens,
        Err(error) => {
            return vec![failure(
                path,
                format!("cannot inspect sealed construction references: {error}"),
            )]
        }
    };
    let mut names = BTreeSet::new();
    collect(tokens, &mut names);
    names
        .into_iter()
        .filter(|name| !permitted(path, name))
        .map(|name| {
            failure(
                path,
                format!("sealed construction name `{name}` may not be referenced from `{path}`"),
            )
        })
        .collect()
}

fn collect(tokens: TokenStream, names: &mut BTreeSet<String>) {
    for token in tokens {
        match token {
            TokenTree::Ident(name) => {
                let spelling = name.to_string();
                let spelling = spelling.strip_prefix("r#").unwrap_or(&spelling);
                if spelling == LOOKUP || BRIDGE.contains(&spelling) {
                    names.insert(spelling.into());
                }
            }
            TokenTree::Group(group) => collect(group.stream(), names),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
