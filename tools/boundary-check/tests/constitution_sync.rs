use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tools directory")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn public_contracts_are_declared_by_the_canonical_machine_constitution() {
    let root = workspace_root();
    let config_text = fs::read_to_string(root.join("tools/boundary-check/config/road1.toml"))
        .expect("read road1.toml");
    let config: toml::Value = toml::from_str(&config_text).expect("parse road1.toml");

    assert_eq!(
        config
            .get("forbidden_root_prefixes")
            .and_then(|value| value.as_array())
            .expect("forbidden_root_prefixes")
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["workspaces/"]
    );
    assert_eq!(
        config
            .get("machine_authority")
            .and_then(|authority| authority.get("mirrored_docs"))
            .and_then(|value| value.as_array())
            .expect("machine_authority.mirrored_docs")
            .len(),
        0,
        "removed private docs must not remain machine authority"
    );

    let born_paths = config
        .get("born_crates")
        .and_then(|value| value.as_array())
        .expect("born_crates")
        .iter()
        .filter_map(|row| row.get("path").and_then(|value| value.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        born_paths,
        [
            "workspaces/worth-contracts/crates/worth-schema-core",
            "workspaces/worth-contracts/crates/worth-schema-graph"
        ]
    );
}

#[test]
fn canonical_config_declares_the_query_audience_framework_family() {
    let root = workspace_root();
    let config_text = fs::read_to_string(root.join("tools/boundary-check/config/road1.toml"))
        .expect("read road1.toml");
    let config: toml::Value = toml::from_str(&config_text).expect("parse road1.toml");
    let query_audience = config
        .get("rule_contracts")
        .and_then(|rules| rules.get("query_audience"))
        .expect("rule_contracts.query_audience");

    assert_eq!(
        query_audience
            .get("engine_package")
            .and_then(|value| value.as_str()),
        Some("worth-query")
    );
    assert_eq!(
        query_audience
            .get("workspace")
            .and_then(|value| value.as_str()),
        Some("workspaces/worth-query")
    );
    assert_eq!(
        query_audience
            .get("engine_package")
            .and_then(|value| value.as_str()),
        Some("worth-query")
    );
    assert_eq!(
        query_audience
            .get("certification_package")
            .and_then(|value| value.as_str()),
        Some("worth-query-certification")
    );
    let audiences = query_audience
        .get("audiences")
        .and_then(|value| value.as_array())
        .expect("audiences array");
    assert_eq!(audiences.len(), 3);

    let packages: Vec<&str> = audiences
        .iter()
        .filter_map(|row| row.get("package").and_then(|value| value.as_str()))
        .collect();
    assert_eq!(
        packages,
        ["worth-query-decl", "worth-query-host", "worth-query-replay"]
    );

    assert!(
        !config_text.contains("query_host_bands"),
        "retired query_host_bands must not remain in road1.toml"
    );
}

#[test]
fn public_manifests_resolve_contracts_without_private_cad_paths() {
    let root = workspace_root();
    for manifest in ["Cargo.toml", "workspaces/worth-query/Cargo.toml"] {
        let text = fs::read_to_string(root.join(manifest)).expect("read public manifest");
        assert!(
            !text.contains("cad/"),
            "{manifest} must not depend on the intentionally removed private tree"
        );
    }
    assert!(root
        .join("workspaces/worth-contracts/crates/worth-schema-graph/Cargo.toml")
        .is_file());
}

#[test]
fn live_automation_and_tooling_have_no_private_cad_dependency() {
    let root = workspace_root();
    let fixture_root = root.join("tools/boundary-check/tests");
    for governed_root in [".github/workflows", "scripts", "tools"] {
        assert_live_tree_has_no_cad_reference(
            &root.join(governed_root),
            &fixture_root,
            governed_root,
        );
    }
}

fn assert_live_tree_has_no_cad_reference(
    path: &std::path::Path,
    fixture_root: &std::path::Path,
    label: &str,
) {
    if path.starts_with(fixture_root)
        || path.file_name().and_then(|name| name.to_str()) == Some("target")
    {
        return;
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("read governed automation or tooling directory") {
            let entry = entry.expect("read governed automation or tooling entry");
            assert_live_tree_has_no_cad_reference(&entry.path(), fixture_root, label);
        }
        return;
    }
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return;
    };
    if !matches!(extension, "rs" | "toml" | "yml" | "yaml" | "sh" | "ps1") {
        return;
    }
    let text = fs::read_to_string(path).expect("read governed automation or tooling file");
    assert!(
        !text.contains("cad/"),
        "{label} live file {} must not depend on the intentionally removed private tree",
        path.display()
    );
}

#[test]
fn worth_proof_law_substrate_row_matches_canonical_naming_sets() {
    let root = workspace_root();
    let config_text = fs::read_to_string(root.join("tools/boundary-check/config/road1.toml"))
        .expect("read road1.toml");
    let config: toml::Value = toml::from_str(&config_text).expect("parse road1.toml");

    let bands = config
        .get("naming")
        .and_then(|naming| naming.get("bands"))
        .and_then(|bands| bands.as_array())
        .expect("naming.bands")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    let substrates = config
        .get("law_substrates")
        .and_then(|value| value.as_array())
        .expect("law_substrates array");
    assert!(
        !substrates.is_empty(),
        "law_substrates must record at least worth-proof"
    );

    let worth_proof = substrates
        .iter()
        .find(|row| row.get("package").and_then(|value| value.as_str()) == Some("worth-proof"))
        .expect("worth-proof law substrate row");

    let tiers = worth_proof
        .get("tiers")
        .and_then(|value| value.as_array())
        .expect("tiers")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    assert_eq!(tiers, ["worth", "worthy"]);

    let substrate_bands = worth_proof
        .get("bands")
        .and_then(|value| value.as_array())
        .expect("bands")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        substrate_bands, bands,
        "worth-proof bands must equal naming.bands exactly"
    );
}
