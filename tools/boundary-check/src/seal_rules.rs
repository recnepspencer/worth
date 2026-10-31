//! Closed caller sets for named test controls and certification factories.
use crate::{
    config::*,
    inputs::Inputs,
    lexical::{close, tokens, Token},
};
use std::collections::{BTreeMap, BTreeSet};
const FACTORIES: &[&str] = &[
    "registered_authoritative_source",
    "with_authoritative_source",
    "from_owner_publication",
    "from_owner_partition_publication",
    "certification_query",
    "readmit_world_home",
    "world_composition_runtime",
    "world_recovery_authority",
    "query_runtime_handle",
    "prepare_home_program",
    "install_home_native_schema",
    "finish_fresh_home_native_readmission",
    "attach_home_registered_source",
    "attach_home_world",
    "native_for_schema_installation",
    "query_finalization_parts",
    "initialize_new_home",
    "query_composition_parts",
    "attach_unrecovered_home",
    "home_schema_installation_runtime",
    "home_query_finalization_parts",
    "query_publication_home_binding",
    "prepare_query_publication_without_signal",
    "bind_query_native_proposal",
    "complete_query_native_preparation",
];
fn owner(input: &Inputs, path: &str) -> Option<String> {
    input
        .manifests
        .iter()
        .filter(|(_, value)| value.get("package").is_some())
        .filter_map(|(p, _)| p.strip_suffix("Cargo.toml"))
        .filter(|p| path.starts_with(p))
        .max_by_key(|p| p.len())
        .map(str::to_owned)
        .or_else(|| path.split_once("/src/").map(|(p, _)| format!("{p}/")))
}
fn test_name(name: &str) -> bool {
    (name.ends_with("_for_test") || name.ends_with("ForTest"))
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
}
fn permitted(path: &str, owners: &BTreeSet<String>, callers: &BTreeSet<(&str, &str)>) -> bool {
    owners.iter().any(|o| {
        if o.ends_with('/') {
            path.starts_with(o)
        } else {
            path == o
        }
    }) || owners.iter().any(|o| callers.contains(&(o.as_str(), path)))
}
fn generic_end(t: &[Token<'_>], open: usize) -> usize {
    let (mut at, mut depth) = (open + 1, 1usize);
    while at < t.len() {
        match t[at].text {
            "<" => depth += 1,
            ">" if t[at - 1].text != "-" => {
                depth -= 1;
                if depth == 0 {
                    return at;
                }
            }
            "(" | "[" | "{" => at = close(t, at),
            _ => {}
        }
        at += 1;
    }
    t.len()
}
// Definition positions establish ownership; every reference is still searched.
fn declarations<'a>(t: &[Token<'a>]) -> (Vec<&'a str>, Vec<&'a str>) {
    let (mut names, mut aliases) = (Vec::new(), Vec::new());
    let mut generic_until = 0;
    for (i, w) in t.windows(2).enumerate() {
        match w[0].text {
            "fn" | "struct" | "enum" | "trait" | "type" | "const" | "static" | "mod" | "union"
            | "macro" | "!" | "use" | "extern" | "impl" => {}
            _ => continue,
        }
        if w[0].end - w[0].start != w[0].text.len() || i > 0 && t[i - 1].text == "'" {
            continue; // Raw identifiers and lifetime names are not item keywords.
        }
        let header = i + if w[0].text == "impl" { 1 } else { 2 };
        if ["fn", "struct", "enum", "trait", "type", "union", "impl"].contains(&w[0].text)
            && t.get(header).is_some_and(|t| t.text == "<")
        {
            generic_until = generic_until.max(generic_end(t, header));
        }
        if w[0].text != "use"
            && !w[1]
                .text
                .chars()
                .next()
                .is_some_and(|c| c == '_' || c.is_alphabetic() || !c.is_ascii())
        {
            continue;
        }
        if w[0].text == "const"
            && (i < generic_until
                || i > 0 && t[i - 1].text == "*"
                || t.get(i + 2).is_none_or(|t| t.text != ":"))
        {
            continue;
        }
        if [
            "fn", "struct", "enum", "trait", "type", "const", "static", "mod", "union", "macro",
        ]
        .contains(&w[0].text)
            || w[0].text == "!" && i > 0 && t[i - 1].text == "macro_rules"
        {
            names.push(if w[0].text == "static" && w[1].text == "mut" {
                t.get(i + 2).map_or("", |t| t.text)
            } else {
                w[1].text
            });
        }
        if ["struct", "enum", "union"].contains(&w[0].text) {
            let mut at = i + 2;
            while at < t.len() && t[at].text != ";" {
                match t[at].text {
                    "<" => at = generic_end(t, at),
                    "{" => {
                        members(t, at, w[0].text == "enum", &mut names);
                        break;
                    }
                    "(" | "[" => at = close(t, at),
                    _ => {}
                }
                at += 1;
            }
        }
        if w[0].text == "use" || w[0].text == "extern" && w[1].text == "crate" {
            let end = (i + 1..t.len())
                .find(|j| t[*j].text == ";")
                .unwrap_or(t.len());
            aliases.extend(
                t[i..end]
                    .windows(2)
                    .filter(|w| w[0].text == "as")
                    .map(|w| w[1].text),
            );
        }
    }
    names.extend(
        t.windows(5)
            .filter(|w| {
                w[0].text == "#"
                    && w[1].text == "["
                    && w[2].text == "proc_macro_derive"
                    && w[3].text == "("
            })
            .map(|w| w[4].text),
    );
    names.retain(|n| test_name(n));
    aliases.retain(|n| test_name(n));
    (names, aliases)
}
fn members<'a>(t: &[Token<'a>], open: usize, variants: bool, names: &mut Vec<&'a str>) {
    let (mut at, mut first) = (open + 1, true);
    while at < close(t, open) {
        if t[at].text == "<" {
            at = generic_end(t, at) + 1;
            continue;
        }
        if t[at].text == "#" && t.get(at + 1).is_some_and(|t| t.text == "[") {
            at = close(t, at + 1) + 1;
            continue;
        }
        if variants && first && t[at].text != "," {
            names.push(t[at].text);
            if t.get(at + 1).is_some_and(|t| t.text == "{") {
                members(t, at + 1, false, names);
            }
            first = false;
        } else if !variants
            && t.get(at + 1).is_some_and(|t| t.text == ":")
            && t.get(at + 2).is_none_or(|t| t.text != ":")
        {
            names.push(t[at].text);
        }
        if t[at].text == "," {
            first = true;
        }
        at = close(t, at) + 1;
    }
}
pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let sources: Vec<_> = input
        .files
        .iter()
        .filter(|(p, s)| {
            p.ends_with(".rs")
                && (s.contains("_for_test")
                    || s.contains("ForTest")
                    || FACTORIES.iter().any(|n| s.contains(n)))
        })
        .map(|(p, s)| (p, tokens(s)))
        .collect();
    let mut doors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut renamed = Vec::new();
    for (path, t) in &sources {
        if let Some(owner) = owner(input, path) {
            let (names, aliases) = declarations(t);
            for name in names {
                doors.entry(name.into()).or_default().insert(owner.clone());
            }
            renamed.extend(aliases.into_iter().map(|name| (name, owner.clone())));
        }
    }
    // Re-exporting an existing name cannot adopt its original owner's door.
    for (name, owner) in renamed {
        doors
            .entry(name.into())
            .or_insert_with(|| BTreeSet::from([owner]));
    }
    let controls: BTreeSet<_> = pairs(input.text(&format!("{CONFIG}control-callers.txt")))
        .into_iter()
        .collect();
    let factories: BTreeSet<_> =
        pairs(input.text(&format!("{CONFIG}certification-construction-callers.txt")))
            .into_iter()
            .collect();
    for (path, t) in &sources {
        let names: BTreeSet<_> = t
            .iter()
            .map(|t| t.text)
            .filter(|name| test_name(name) || FACTORIES.contains(name))
            .collect();
        for name in &names {
            if let Some(owners) = doors.get(*name)
                && !permitted(path, owners, &controls)
            {
                out.push(diagnostic(
                    Code::Control,
                    path,
                    format!("control `{name}` requires its owner or an exact caller"),
                ));
            }
            if FACTORIES.contains(name) {
                let (class, owner) = if *name == "world_recovery_authority" {
                    (
                        "native-world-readmission",
                        "workspaces/worth-store/crates/worth-store-relational/",
                    )
                } else if *name == "world_composition_runtime" {
                    (
                        "native-home-composition",
                        "workspaces/worth-store/crates/worth-store-relational/",
                    )
                } else if *name == "query_runtime_handle" {
                    ("query-registered-native", "crates/worth-runtime-bridge/")
                } else if matches!(
                    *name,
                    "prepare_home_program"
                        | "install_home_native_schema"
                        | "finish_fresh_home_native_readmission"
                        | "attach_home_registered_source"
                        | "attach_home_world"
                ) {
                    (
                        "query-home-preparation",
                        "workspaces/worth-query/crates/worth-query-execution/",
                    )
                } else if matches!(
                    *name,
                    "native_for_schema_installation"
                        | "query_finalization_parts"
                        | "initialize_new_home"
                        | "query_composition_parts"
                ) {
                    (
                        "native-home-query-preparation",
                        "workspaces/worth-store/crates/worth-store-application/",
                    )
                } else if *name == "attach_unrecovered_home" {
                    (
                        "native-home-attachment",
                        "workspaces/worth-store/crates/worth-store-relational/",
                    )
                } else if *name == "home_schema_installation_runtime" {
                    (
                        "native-home-schema-installation",
                        "workspaces/worth-store/crates/worth-store-relational/",
                    )
                } else if *name == "home_query_finalization_parts" {
                    (
                        "native-home-query-finalization",
                        "workspaces/worth-store/crates/worth-store-relational/",
                    )
                } else if matches!(
                    *name,
                    "query_publication_home_binding"
                        | "prepare_query_publication_without_signal"
                        | "bind_query_native_proposal"
                        | "complete_query_native_preparation"
                ) {
                    (
                        "query-world-commit-preparation",
                        "crates/worth-runtime-world/",
                    )
                } else if *name == "readmit_world_home" {
                    ("world-home", "crates/worth-runtime-world/")
                } else if *name == "certification_query" {
                    (
                        "installation",
                        "workspaces/worth-query/crates/worth-query-installation/",
                    )
                } else {
                    ("bridge", "crates/worth-runtime-bridge/")
                };
                if !path.starts_with(owner) && !factories.contains(&(class, path.as_str())) {
                    out.push(diagnostic(
                        Code::Construction,
                        path,
                        format!("factory `{name}` requires its owner or an exact caller"),
                    ));
                }
            }
        }
    }
    out
}
#[cfg(test)]
mod tests;
