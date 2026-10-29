//! Closed, area-owned exceptions to one court per product frontier.
use super::{
    failure,
    product_frontier::{self, Frontier},
    structure_limits,
    workspace_members::Member,
};
use crate::diagnostics::Diagnostic;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub(super) const CONFIG: &str = "tools/boundary-check/config/court-frontier-splits.toml";

#[derive(Default, Deserialize)]
struct Configuration {
    #[serde(default)]
    splits: Vec<Split>,
}
#[derive(Deserialize)]
struct Split {
    court: String,
    frontier: Frontier,
    area: String,
    owners: Vec<String>,
    #[serde(default)]
    delegates: Vec<String>,
}

pub(super) fn validate(
    root: &Path,
    members: &BTreeMap<PathBuf, (String, Member)>,
) -> Result<Vec<Diagnostic>, String> {
    if !root.join(CONFIG).exists()
        && !members
            .values()
            .any(|(_, member)| member.frontier.is_some())
    {
        return Ok(Vec::new());
    }
    let config = match std::fs::read_to_string(root.join(CONFIG)) {
        Ok(text) => toml::from_str::<Configuration>(&text).map_err(|e| format!("{CONFIG}: {e}"))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && !root.join(".git").exists() => {
            Configuration::default()
        }
        Err(e) => return Err(format!("{CONFIG}: {e}")),
    };
    let mut registrations = BTreeMap::new();
    for split in &config.splits {
        if split.area.is_empty()
            || split.owners.is_empty()
            || !split
                .area
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(format!(
                "{CONFIG}: a ceiling split needs a named area and owner selectors"
            ));
        }
        for selector in split.owners.iter().chain(&split.delegates) {
            if selector.is_empty()
                || selector.split('/').any(|s| {
                    s.is_empty()
                        || s == "."
                        || s == ".."
                        || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                })
            {
                return Err(format!("{CONFIG}: invalid owner selector {selector}"));
            }
        }
        if registrations.insert(split.court.as_str(), split).is_some() {
            return Err(format!("{CONFIG}: duplicate court {}", split.court));
        }
    }
    let mut groups: BTreeMap<_, Vec<_>> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let mut found = BTreeSet::new();
    for (manifest, (stem, member)) in members {
        let Some(frontier) = &member.frontier else {
            continue;
        };
        let text = crate::source_documents::read_to_string(manifest).map_err(|e| e.to_string())?;
        let value: toml::Value = toml::from_str(&text).map_err(|e| e.to_string())?;
        let name = value["package"]["name"]
            .as_str()
            .ok_or("member has no package name")?;
        let base = format!("{stem}-court-{}", product_frontier::surface(stem, frontier));
        let split = registrations.get(name);
        if let Some(split) = split {
            found.insert(name.to_owned());
            if split.frontier != *frontier {
                diagnostics.push(failure(
                    CONFIG,
                    format!("{} registers a different product frontier", split.court),
                ));
            }
        }
        let expected = split.map_or_else(
            || base.clone(),
            |s| {
                if name == base {
                    base.clone()
                } else {
                    format!("{base}-{}", s.area)
                }
            },
        );
        if name != expected {
            diagnostics.push(failure(&manifest.to_string_lossy(), format!("court name does not match product frontier {frontier:?}; expected `{expected}`")));
        }
        groups
            .entry((stem, frontier))
            .or_default()
            .push((name.to_owned(), manifest, split));
    }
    for ((_, frontier), courts) in groups {
        let names: Vec<_> = courts.iter().map(|(name, _, _)| name.as_str()).collect();
        let registered: Vec<_> = courts.iter().filter_map(|(_, _, split)| *split).collect();
        if courts.len() == 1 && !registered.is_empty() {
            diagnostics.push(failure(
                CONFIG,
                format!("{} has no equal-frontier ceiling sibling", names[0]),
            ));
        }
        if courts.len() < 2 {
            continue;
        }
        let mut lines = 0;
        for (_, manifest, _) in &courts {
            lines += structure_limits::rust_lines(manifest.parent().unwrap())?;
        }
        let areas: BTreeSet<_> = registered.iter().map(|s| s.area.as_str()).collect();
        if registered.len() != courts.len()
            || areas.len() != courts.len()
            || lines <= structure_limits::CEILING
        {
            diagnostics.push(failure(CONFIG, format!("equal product frontiers {frontier:?} in {names:?}; merge courts unless registered area splits are required by the {}-line ceiling (combined {lines})", structure_limits::CEILING)));
            continue;
        }
        for split in &registered {
            for delegate in &split.delegates {
                if !split.owners.iter().any(|owner| {
                    registered.iter().any(|other| {
                        other.court != split.court
                            && other.owners.iter().any(|child| {
                                child == &format!("{owner}/{delegate}")
                                    || child.starts_with(&format!("{owner}/{delegate}/"))
                            })
                    })
                }) {
                    diagnostics.push(failure(
                        CONFIG,
                        format!(
                            "{} delegates {delegate} without a registered owning area",
                            split.court
                        ),
                    ));
                }
            }
            for other in &registered {
                if split.court == other.court {
                    continue;
                }
                for owner in &split.owners {
                    for child in &other.owners {
                        if child == owner
                            || (child.starts_with(&format!("{owner}/"))
                                && !split.delegates.iter().any(|delegate| {
                                    child == &format!("{owner}/{delegate}")
                                        || child.starts_with(&format!("{owner}/{delegate}/"))
                                }))
                        {
                            diagnostics.push(failure(
                                CONFIG,
                                format!(
                                    "overlapping area owners in {} and {}",
                                    split.court, other.court
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    for name in registrations.keys().filter(|name| !found.contains(**name)) {
        diagnostics.push(failure(
            CONFIG,
            format!("ceiling split names an absent court: {name}"),
        ));
    }
    Ok(diagnostics)
}
