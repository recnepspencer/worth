use super::*;

#[test]
fn new_manifest_and_source_flags_fail_the_one_build_rule() {
    let path = "workspaces/worth-query/crates/new-owner/Cargo.toml";
    let diagnostics = manifest(path, "[features]\nnew_test_switch = []\n");
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message().contains("new_test_switch"));
    let diagnostics = source_predicates("workspaces/worth-query/crates/new-owner/src/lib.rs",
        "#[cfg_attr(feature = \"new_test_switch\", derive(Debug))] struct X; fn f() { if cfg!(not(feature = \"another_switch\")) {} }");
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.subject().ends_with("src/lib.rs")));
}

#[test]
fn only_ruled_owner_specific_exceptions_pass() {
    assert!(manifest("crates/worth-signal/Cargo.toml", "[features]\ndefault = [\"profile-standard\"]\nprofile-standard = []\ntest-peak-allocation = []").is_empty());
    assert!(source_predicates(
        "crates/worth-signal/src/lib.rs",
        "#[cfg(feature = \"test-peak-allocation\")] struct Peak;"
    )
    .is_empty());
    assert!(manifest("crates/worth-signal-wasm/Cargo.toml", "[features]\ndefault = [\"profile-standard\"]\nprofile-standard = [\"worth-signal/profile-standard\"]").is_empty());
    assert_eq!(
        manifest(
            "crates/worth-signal-wasm/Cargo.toml",
            "[features]\ntest-peak-allocation = []"
        )
        .len(),
        1
    );
    assert_eq!(
        source_predicates(
            "crates/worth-runtime-world/src/lib.rs",
            "#[cfg(feature = \"profile-standard\")] struct X;"
        )
        .len(),
        1
    );
}

#[test]
fn comments_literals_and_ordinary_identifiers_are_not_predicates() {
    assert!(source_predicates(
        "crates/worth-runtime-world/src/lib.rs",
        r#"// #[cfg(feature = "gone")]
const TEXT: &str = "feature = gone"; fn f() { let feature = "gone"; }"#
    )
    .is_empty());
}

#[test]
fn implicit_features_and_target_gates_cannot_reopen_two_builds() {
    assert_eq!(manifest("crates/worth-relational/Cargo.toml", "[dependencies]\nprobe = { version = \"1\", optional = true }\n[[test]]\nname = \"hidden\"\nrequired-features = [\"probe\"]").len(), 2);
    assert_eq!(
        source_predicates(
            "crates/worth-execution/src/lib.rs",
            "macro_rules! hidden { () => { #[cfg(all(test, feature = \"probe\"))] fn f() {} } }"
        )
        .len(),
        1
    );
}

#[test]
fn an_injected_manifest_and_hidden_source_fail_repository_validation() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temporary = std::env::temp_dir().canonicalize().unwrap();
    let root = temporary.join(format!("worth-one-build-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(root.join("workspaces/future-team")).unwrap();
    let owner = root.join("workspaces/future-team");
    std::fs::write(
        owner.join("Cargo.toml"),
        "[features]\nfuture_test_build = []\n",
    )
    .unwrap();
    std::fs::create_dir(owner.join("src")).unwrap();
    std::fs::write(
        owner.join("src/lib.rs"),
        "#[cfg(feature = \"future_test_build\")] fn hidden() {}",
    )
    .unwrap();
    let diagnostics = validate(&root);
    let resolved = root.canonicalize().unwrap();
    assert!(resolved.starts_with(&temporary) && resolved != temporary);
    std::fs::remove_dir_all(resolved).unwrap();
    assert_eq!(diagnostics.len(), 2);
    for path in [
        "workspaces/future-team/Cargo.toml",
        "workspaces/future-team/src/lib.rs",
    ] {
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.subject() == path));
    }
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.message().contains("future_test_build")));
}

#[test]
fn raw_cfg_and_feature_identifiers_cannot_hide_a_predicate() {
    assert_eq!(
        source_predicates(
            "crates/worth-execution/src/lib.rs",
            "#[r#cfg(r#feature = \"probe\")] fn hidden() {}"
        )
        .len(),
        1
    );
}

#[test]
fn empty_feature_tables_are_refused() {
    let diagnostics = manifest("crates/new-owner/Cargo.toml", "[features]\n");
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message().contains("empty features table"));
}

#[test]
fn governed_dependency_features_are_refused_even_through_workspace_aliases() {
    let owners = OwnedDependencies::from_manifests(&[
        ("crates/new-owner/Cargo.toml".into(), "[package]\nname = \"new-owner\"\n".into()),
        ("Cargo.toml".into(), "[workspace.dependencies]\nrenamed = { package = \"new-owner\", path = \"crates/new-owner\" }\n".into()),
    ]);
    for source in [
        "[dependencies]\nnew-owner = { path = \"../new-owner\", features = [] }\n",
        "[dev-dependencies]\nalias = { package = \"new-owner\", path = \"../new-owner\", features = [\"future\"] }\n",
        "[target.'cfg(windows)'.build-dependencies]\nrenamed = { workspace = true, features = [\"future\"] }\n",
    ] {
        let diagnostics = manifest_with_owners("crates/consumer/Cargo.toml", source, &owners);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message().contains("dependency features"));
    }
    assert!(manifest_with_owners(
        "crates/consumer/Cargo.toml",
        "[dependencies]\nserde = { version = \"1\", features = [\"derive\"] }\n",
        &owners
    )
    .is_empty());
}

#[test]
fn only_named_other_team_roots_are_excluded() {
    for path in [
        "workspaces/worth-store/Cargo.toml",
        "workspaces/worth-ui/src/lib.rs",
        "crates/worth-proof/Cargo.toml",
    ] {
        assert!(!governed(path));
    }
    for path in [
        "workspaces/future-team/Cargo.toml",
        "crates/worth-proof-successor/Cargo.toml",
        "tools/new-owner/Cargo.toml",
    ] {
        assert!(governed(path));
    }
}
