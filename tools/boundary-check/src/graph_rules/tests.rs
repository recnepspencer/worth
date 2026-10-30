use super::*;
#[test]
fn courts_have_one_canonical_target_and_remain_dependency_leaves() {
    let directory = "workspaces/a/crates/a-court-source";
    let manifest = format!("{directory}/Cargo.toml");
    let test_source = format!("{directory}/tests/a_court_source.rs");
    let mut input = Inputs::fixture(&[(&manifest, "[package]\nname='a-court-source'\nautotests=false\n[[test]]\nname='a_court_source'\npath='tests/a_court_source.rs'"), (&test_source, "#[test] fn succeeds() {}")], "");
    input.files.insert(
        format!("{CONFIG}test-workspaces.txt"),
        "workspaces/a | a".into(),
    );
    input
        .files
        .insert(format!("{CONFIG}court-frontier-splits.toml"), "".into());
    let court = Package {
        id: "court".into(),
        name: "a-court-source".into(),
        manifest_path: manifest.clone(),
        targets: vec![Target {
            name: "a_court_source".into(),
            kind: vec!["test".into()],
            src_path: format!("{directory}/tests/a_court_source.rs"),
        }],
        ..Default::default()
    };
    input.graphs.insert(
        "workspaces/a/Cargo.toml".into(),
        Metadata {
            packages: vec![court],
            workspace_members: vec!["court".into()],
            ..Default::default()
        },
    );
    assert!(check(&input).is_empty());
    input
        .graphs
        .get_mut("workspaces/a/Cargo.toml")
        .unwrap()
        .packages[0]
        .targets
        .clear();
    assert!(check(&input)
        .iter()
        .any(|d| d.message.contains("canonical")));
    input
        .graphs
        .get_mut("workspaces/a/Cargo.toml")
        .unwrap()
        .packages[0]
        .dependencies
        .push(Dependency {
            name: "other-court-root".into(),
            ..Default::default()
        });
    assert!(check(&input).iter().any(|d| d.message.contains("leaf")));
}
#[test]
fn products_cannot_hide_integration_tests_in_the_file_list() {
    let manifest = "workspaces/a/crates/product/Cargo.toml";
    let mut input = Inputs::fixture(&[(manifest, "[package]\nname='product'")], "");
    input.files.insert(
        format!("{CONFIG}test-workspaces.txt"),
        "workspaces/a | a".into(),
    );
    input
        .files
        .insert(format!("{CONFIG}court-frontier-splits.toml"), "".into());
    input.graphs.insert(
        "workspaces/a/Cargo.toml".into(),
        Metadata {
            packages: vec![Package {
                id: "product".into(),
                name: "product".into(),
                manifest_path: manifest.into(),
                ..Default::default()
            }],
            workspace_members: vec!["product".into()],
            ..Default::default()
        },
    );
    assert!(check(&input).is_empty());
    input
        .paths
        .insert("workspaces/a/crates/product/tests/hidden.rs".into());
    assert!(check(&input).iter().any(|d| d.code == "BC9002"));
}
#[test]
fn dependency_denials_and_allowlists_reject_only_disallowed_edges() {
    let p = Package {
        name: "source".into(),
        dependencies: vec![Dependency {
            name: "target".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    for (key, policy) in [
        (
            "dependency_denials",
            "sources=['source']\nforbidden_targets=['target']",
        ),
        (
            "source_dependency_allowlists",
            "sources=['source']\nallowed_targets=[]",
        ),
        (
            "dependency_target_allowlists",
            "governed_source_prefixes=['source']\ntarget='target'\nallowed_sources=[]",
        ),
    ] {
        let rule = toml::from_str(policy).unwrap();
        assert_eq!(dependencies(key, &rule, &p).len(), 1);
        let good = Package {
            name: "elsewhere".into(),
            ..Default::default()
        };
        assert!(dependencies(key, &rule, &good).is_empty());
    }
}
#[test]
fn categories_follow_top_level_directory_segments() {
    assert!(category_valid("process/serial/settlement.rs"));
    assert!(!category_valid("settlement/process/actor.rs"));
    assert!(!category_valid("fast/settlement.rs"));
}
#[test]
fn dependency_contracts_preserve_exact_versions_defaults_and_features() {
    let rule = toml::from_str("workspace_manifest='Cargo.toml'\nsources=['source']\nallowed_targets=['target']\nguidance='exact admission'\n[[dependency_contracts]]\ntarget='target'\nversion_requirement='=0.2.21'\nuses_default_features=true\nfeatures=[]").unwrap();
    for (req, defaults, features, denied) in [
        ("=0.2.21", true, vec![], false),
        ("^0.2.21", true, vec![], true),
        ("=0.2.21", false, vec![], true),
        ("=0.2.21", true, vec!["extra"], true),
    ] {
        let package = Package {
            name: "source".into(),
            dependencies: vec![Dependency {
                name: "target".into(),
                req: req.into(),
                uses_default_features: defaults,
                features: features.into_iter().map(str::to_owned).collect(),
            }],
            ..Default::default()
        };
        assert_eq!(
            !dependencies("source_dependency_allowlists", &rule, &package).is_empty(),
            denied
        );
    }
}
