//! Product compile-closure frontiers and their canonical naming.
use crate::config::*;
use std::collections::{BTreeMap, BTreeSet};
fn closure(id: &str, dev: bool, nodes: &BTreeMap<&str, &Node>) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![(id, dev)];
    while let Some((id, dev)) = pending.pop() {
        if let Some(node) = nodes.get(id) {
            for edge in &node.deps {
                if (dev
                    || edge
                        .dep_kinds
                        .iter()
                        .any(|k| k.kind.as_deref() != Some("dev")))
                    && seen.insert(edge.pkg.clone())
                {
                    pending.push((&edge.pkg, false));
                }
            }
        }
    }
    seen
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frontier_uses_reachable_product_roots_and_excludes_dependency_dev_edges() {
        let p = |id: &str| Package {
            id: id.into(),
            name: id.into(),
            ..Default::default()
        };
        let edge = |id: &str, dev: bool| Edge {
            pkg: id.into(),
            dep_kinds: vec![Kind {
                kind: dev.then(|| "dev".into()),
            }],
        };
        let graph = Metadata {
            packages: vec![p("host"), p("engine"), p("unrelated"), p("a-court-host")],
            workspace_members: vec![
                "host".into(),
                "engine".into(),
                "unrelated".into(),
                "a-court-host".into(),
            ],
            resolve: Some(Resolve {
                nodes: vec![
                    Node {
                        id: "a-court-host".into(),
                        deps: vec![edge("host", true)],
                    },
                    Node {
                        id: "host".into(),
                        deps: vec![edge("engine", false), edge("unrelated", true)],
                    },
                    Node {
                        id: "engine".into(),
                        deps: vec![],
                    },
                ],
            }),
        };
        assert_eq!(frontiers(&graph)["a-court-host"], vec!["host"]);
        assert_eq!(surface("a", &["host".into()]), "host");
        assert_ne!(surface("a", &["engine".into()]), "host");
    }
}
pub(super) fn frontiers(graph: &Metadata) -> BTreeMap<String, Vec<String>> {
    let nodes: BTreeMap<_, _> = graph
        .resolve
        .iter()
        .flat_map(|r| &r.nodes)
        .map(|n| (n.id.as_str(), n))
        .collect();
    let products: Vec<_> = graph
        .packages
        .iter()
        .filter(|p| {
            graph.workspace_members.contains(&p.id)
                && !p.name.contains("-court-")
                && !p.name.contains("-testkit")
                && !p.manifest_path.contains("/fixtures/")
        })
        .collect();
    let dependencies: BTreeMap<_, _> = products
        .iter()
        .map(|p| (&p.id, closure(&p.id, false, &nodes)))
        .collect();
    graph
        .packages
        .iter()
        .filter(|p| p.name.contains("-court-") && graph.workspace_members.contains(&p.id))
        .map(|court| {
            let reached = closure(&court.id, true, &nodes);
            let selected: Vec<_> = products
                .iter()
                .filter(|p| reached.contains(&p.id))
                .collect();
            let mut frontier: Vec<_> = selected
                .iter()
                .filter(|p| {
                    !selected
                        .iter()
                        .any(|other| other.id != p.id && dependencies[&other.id].contains(&p.id))
                })
                .map(|p| p.name.clone())
                .collect();
            frontier.sort();
            (court.id.clone(), frontier)
        })
        .collect()
}
pub(super) fn surface(stem: &str, frontier: &[String]) -> String {
    if frontier.is_empty() {
        return "source".into();
    }
    let prefix = stem.strip_prefix("worth-query-").unwrap_or(stem);
    frontier
        .iter()
        .map(|p| {
            p.strip_prefix(&format!("{prefix}-")).unwrap_or_else(|| {
                if p == prefix {
                    p.rsplit('-').next().unwrap()
                } else {
                    p
                }
            })
        })
        .collect::<Vec<_>>()
        .join("-")
}
