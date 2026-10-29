use super::*;
use crate::manifest_types::{
    CargoMetadataDependencyKind, CargoMetadataPackage, CargoMetadataResolve,
    CargoMetadataResolvedDependency,
};

fn graph() -> CargoMetadata {
    let rows: Vec<(&str, Vec<(&str, Option<&str>)>)> = vec![
        ("sample-decl", vec![]),
        (
            "sample-host",
            vec![("sample-decl", None), ("sample-extra", Some("dev"))],
        ),
        ("sample-extra", vec![]),
        (
            "sample-testkit-a",
            vec![("sample-host", None), ("serde_json", None)],
        ),
        (
            "sample-testkit-b",
            vec![("sample-host", None), ("tokio", None)],
        ),
        ("sample-court-one", vec![("sample-testkit-a", Some("dev"))]),
        ("sample-court-two", vec![("sample-testkit-b", None)]),
        ("sample-court-source", vec![]),
        ("serde_json", vec![]),
        ("tokio", vec![]),
    ];
    CargoMetadata {
        workspace_members: rows
            .iter()
            .filter(|(name, _)| name.starts_with("sample-"))
            .map(|(name, _)| (*name).into())
            .collect(),
        packages: rows
            .iter()
            .map(|(name, _)| CargoMetadataPackage {
                id: (*name).into(),
                name: (*name).into(),
                manifest_path: format!("/repository/{name}/Cargo.toml"),
                dependencies: Vec::new(),
            })
            .collect(),
        resolve: Some(CargoMetadataResolve {
            nodes: rows
                .iter()
                .map(|(name, deps)| CargoMetadataNode {
                    id: (*name).into(),
                    deps: deps
                        .iter()
                        .map(|(name, kind)| CargoMetadataResolvedDependency {
                            pkg: (*name).into(),
                            dep_kinds: vec![CargoMetadataDependencyKind {
                                kind: kind.map(str::to_owned),
                            }],
                        })
                        .collect(),
                })
                .collect(),
        }),
    }
}

#[test]
fn kits_and_external_dependencies_do_not_distinguish_product_frontiers() {
    let frontiers = compute(&graph()).unwrap();
    assert_eq!(
        frontiers["sample-court-one"],
        Frontier::from(["sample-host".into()])
    );
    assert_eq!(frontiers["sample-court-one"], frontiers["sample-court-two"]);
}

#[test]
fn dependency_crates_private_dev_tests_do_not_expand_the_compile_frontier() {
    let frontiers = compute(&graph()).unwrap();
    assert!(!frontiers["sample-court-one"].contains("sample-extra"));
    assert!(!frontiers["sample-court-one"].contains("sample-decl"));
}

#[test]
fn fixtures_are_traversed_without_becoming_product_frontier_identities() {
    let mut metadata = graph();
    metadata
        .packages
        .iter_mut()
        .find(|p| p.name == "sample-host")
        .unwrap()
        .manifest_path = "/repository/fixtures/sample-host/Cargo.toml".into();
    let frontiers = compute(&metadata).unwrap();
    assert_eq!(
        frontiers["sample-court-one"],
        Frontier::from(["sample-decl".into()])
    );
}

#[test]
fn source_only_courts_and_owning_product_prefixes_have_canonical_names() {
    let frontiers = compute(&graph()).unwrap();
    assert_eq!(
        surface("sample", &frontiers["sample-court-source"]),
        "source"
    );
    assert_eq!(
        surface(
            "worth-query",
            &Frontier::from(["worth-query".into(), "worth-query-host".into()])
        ),
        "query-host"
    );
    assert_eq!(
        surface(
            "worth-query-bank",
            &Frontier::from(["bank-external-rail".into(), "bank-server".into()])
        ),
        "external-rail-server"
    );
}

#[test]
fn a_missing_resolved_graph_is_a_configuration_error() {
    let mut metadata = graph();
    metadata.resolve = None;
    assert!(compute(&metadata)
        .unwrap_err()
        .contains("no resolved graph"));
}
