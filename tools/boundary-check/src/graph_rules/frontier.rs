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
                && !p.manifest_path.split('/').any(|s| s == "fixtures")
        })
        .collect();
    // Keep the combined court convention: traverse infrastructure, and retain
    // established leaf APIs while pruning incidental clients from the frontier.
    let established: BTreeSet<_> = select(graph, &products, &nodes)
        .into_iter()
        .filter(|(id, frontier)| {
            let court = graph.packages.iter().find(|p| &p.id == id).unwrap();
            let (stem, label) = court.name.split_once("-court-").unwrap();
            let surface = surface(stem, frontier);
            label == surface || label.starts_with(&format!("{surface}-"))
        })
        .flat_map(|(_, frontier)| frontier)
        .collect();
    let mut products = products;
    products.retain(|p| {
        !p.manifest_path.split('/').any(|s| s == "tools")
            && !(p.name.ends_with("-runner")
                && !p
                    .targets
                    .iter()
                    .any(|t| t.kind.iter().any(|k| k == "lib" || k == "rlib")))
            && !["-test-support", "-certification", "-formal-models"]
                .iter()
                .any(|part| p.name.contains(part))
            && !p.name.ends_with("-harness")
    });
    let mut incoming: BTreeMap<&str, Vec<(&str, &Edge)>> = BTreeMap::new();
    for node in nodes.values() {
        for edge in &node.deps {
            incoming
                .entry(&edge.pkg)
                .or_default()
                .push((&node.id, edge));
        }
    }
    loop {
        let ids: BTreeSet<_> = products.iter().map(|p| p.id.as_str()).collect();
        let incidental: BTreeSet<_> = products
            .iter()
            .filter(|p| !established.contains(&p.name))
            .filter(|p| {
                incoming.get(p.id.as_str()).is_some_and(|edges| {
                    edges.iter().all(|(caller, edge)| {
                        (graph
                            .workspace_members
                            .iter()
                            .any(|id| id.as_str() == *caller)
                            && !ids.contains(caller))
                            || edge
                                .dep_kinds
                                .iter()
                                .all(|k| k.kind.as_deref() == Some("dev"))
                    })
                })
            })
            .map(|p| p.id.clone())
            .collect();
        if incidental.is_empty() {
            break;
        }
        products.retain(|p| !incidental.contains(&p.id));
    }
    select(graph, &products, &nodes)
}
fn select(
    graph: &Metadata,
    products: &[&Package],
    nodes: &BTreeMap<&str, &Node>,
) -> BTreeMap<String, Vec<String>> {
    let dependencies: BTreeMap<_, _> = products
        .iter()
        .map(|p| (&p.id, closure(&p.id, false, nodes)))
        .collect();
    graph
        .packages
        .iter()
        .filter(|p| p.name.contains("-court-") && graph.workspace_members.contains(&p.id))
        .map(|court| {
            let reached = closure(&court.id, true, nodes);
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

#[cfg(test)]
mod tests {
    use super::*;
    fn graph() -> Metadata {
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
        Metadata {
            packages: vec![
                p("host"),
                p("engine"),
                p("unrelated"),
                p("a-court-host"),
                p("a-court-engine"),
            ],
            workspace_members: vec![
                "host".into(),
                "engine".into(),
                "unrelated".into(),
                "a-court-host".into(),
                "a-court-engine".into(),
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
                    Node {
                        id: "a-court-engine".into(),
                        deps: vec![edge("engine", false)],
                    },
                ],
            }),
        }
    }
    #[test]
    fn frontier_uses_reachable_product_roots_and_excludes_dependency_dev_edges() {
        let graph = graph();
        assert_eq!(frontiers(&graph)["a-court-host"], vec!["host"]);
        assert_eq!(surface("a", &["host".into()]), "host");
        assert_ne!(surface("a", &["engine".into()]), "host");
    }
    #[test]
    fn infrastructure_and_incidental_clients_are_traversed_to_established_products() {
        let mut graph = graph();
        // This incumbent keeps the host's established public frontier while the
        // other court also reaches tooling and clients through fixture support.
        graph.workspace_members.push("a-court-host-existing".into());
        graph.packages.push(Package {
            id: "a-court-host-existing".into(),
            name: "a-court-host-existing".into(),
            ..Default::default()
        });
        graph.resolve.as_mut().unwrap().nodes.push(Node {
            id: "a-court-host-existing".into(),
            deps: vec![Edge {
                pkg: "host".into(),
                dep_kinds: vec![Kind { kind: None }],
            }],
        });
        for name in [
            "a-test-support",
            "a-certification",
            "a-formal-models",
            "a-harness",
            "a-reporter",
            "a-runner",
            "a-incidental",
        ] {
            graph.workspace_members.push(name.into());
            graph.packages.push(Package {
                id: name.into(),
                name: name.into(),
                manifest_path: if name == "a-reporter" {
                    "tools/reporter/Cargo.toml".into()
                } else {
                    format!("crates/{name}/Cargo.toml")
                },
                targets: vec![Target {
                    kind: vec!["bin".into()],
                    ..Default::default()
                }],
                ..Default::default()
            });
            let resolve = graph.resolve.as_mut().unwrap();
            resolve.nodes.push(Node {
                id: name.into(),
                deps: vec![Edge {
                    pkg: "host".into(),
                    dep_kinds: vec![Kind { kind: None }],
                }],
            });
            resolve
                .nodes
                .iter_mut()
                .find(|n| n.id == "a-court-host")
                .unwrap()
                .deps
                .push(Edge {
                    pkg: name.into(),
                    dep_kinds: vec![Kind { kind: None }],
                });
        }
        let result = frontiers(&graph);
        assert_eq!(result["a-court-host"], vec!["host"]);
        assert_eq!(result["a-court-host-existing"], vec!["host"]);
    }
    #[test]
    fn fixtures_are_traversed_without_becoming_product_frontiers() {
        let mut graph = graph();
        graph
            .packages
            .iter_mut()
            .find(|p| p.id == "host")
            .unwrap()
            .manifest_path = "fixtures/host/Cargo.toml".into();
        assert_eq!(frontiers(&graph)["a-court-host"], vec!["engine"]);
    }
}
