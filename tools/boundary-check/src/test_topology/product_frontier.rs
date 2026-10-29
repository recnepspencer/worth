//! Compile-closure product roots; incidental libraries and fixtures do not split courts.
use crate::manifest_types::{CargoMetadata, CargoMetadataNode};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Frontier = BTreeSet<String>;

pub(super) fn compute(metadata: &CargoMetadata) -> Result<BTreeMap<String, Frontier>, String> {
    let resolve = metadata
        .resolve
        .as_ref()
        .ok_or("Cargo metadata has no resolved graph")?;
    let nodes: BTreeMap<_, _> = resolve.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let members: BTreeSet<_> = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect();
    let products: BTreeMap<_, _> = metadata
        .packages
        .iter()
        .filter(|p| {
            members.contains(p.id.as_str())
                && !p.name.contains("-court-")
                && !p.name.contains("-testkit")
                && !p
                    .manifest_path
                    .replace('\\', "/")
                    .split('/')
                    .any(|s| s == "fixtures")
        })
        .map(|p| (p.id.as_str(), p.name.as_str()))
        .collect();
    let dependencies: BTreeMap<_, _> = products
        .keys()
        .map(|id| Ok((*id, closure(id, false, &nodes)?)))
        .collect::<Result<_, String>>()?;
    let mut result = BTreeMap::new();
    for court in metadata
        .packages
        .iter()
        .filter(|p| members.contains(p.id.as_str()) && p.name.contains("-court-"))
    {
        let reached = closure(&court.id, true, &nodes)?;
        let selected: BTreeSet<_> = products
            .keys()
            .copied()
            .filter(|id| reached.contains(*id))
            .collect();
        let frontier = selected
            .iter()
            .filter(|id| {
                !selected
                    .iter()
                    .any(|other| other != *id && dependencies[other].contains(**id))
            })
            .map(|id| products[id].to_owned())
            .collect();
        result.insert(court.id.clone(), frontier);
    }
    Ok(result)
}

fn closure<'a>(
    start: &'a str,
    court: bool,
    nodes: &BTreeMap<&'a str, &'a CargoMetadataNode>,
) -> Result<BTreeSet<&'a str>, String> {
    let mut reached = BTreeSet::new();
    let mut pending = vec![(start, court)];
    while let Some((id, include_dev)) = pending.pop() {
        let node = nodes
            .get(id)
            .ok_or_else(|| format!("resolved node is missing: {id}"))?;
        for edge in &node.deps {
            // Dependency crates' own dev tests are not compiled by this court.
            if (include_dev
                || edge
                    .dep_kinds
                    .iter()
                    .any(|k| k.kind.as_deref() != Some("dev")))
                && reached.insert(edge.pkg.as_str())
            {
                pending.push((edge.pkg.as_str(), false));
            }
        }
    }
    Ok(reached)
}

pub(super) fn surface(stem: &str, frontier: &Frontier) -> String {
    if frontier.is_empty() {
        return "source".into();
    }
    let product_prefix = stem.strip_prefix("worth-query-").unwrap_or(stem);
    frontier
        .iter()
        .map(|name| {
            name.strip_prefix(&format!("{product_prefix}-"))
                .unwrap_or_else(|| {
                    if name == product_prefix {
                        name.rsplit('-').next().unwrap()
                    } else {
                        name
                    }
                })
        })
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests;
