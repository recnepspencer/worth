//! The normal CLI scan must execute consumer-document enforcement.
use crate::query_audience_fixture::{
    base_config, run_boundary_check, unique_temp_root, write_file, write_query_stubs,
    write_root_shell,
};

#[test]
fn consumer_fence_debt_is_enforced_by_the_production_scan() {
    let root = unique_temp_root("consumer-doc-scan");
    write_query_stubs(&root);
    write_root_shell(
        &root,
        &base_config(
            "",
            "",
            "[[naming.reserved_domains]]\ntier='worth'\nband='schema'\ndomains=['core']\n",
            "[[rule_contracts.band_rules]]\nsource_band='schema'\nallowed_target_bands=[]\n",
        ),
    );
    write_file(&root, "docs/guide.md", "```rust\nlet x = 1;\n```\n");
    write_file(
        &root,
        "tools/boundary-check/config/consumer-docs.toml",
        "documents = ['docs/guide.md']\ndoctests = []\n",
    );
    write_file(
        &root,
        "tools/boundary-check/snapshots/consumer-doc-fence-debt.toml",
        "[[documents]]\npath = 'docs/guide.md'\nunclassified = 1\n",
    );
    let (ok, output) = run_boundary_check(&root);
    assert!(ok, "the valid baseline must pass the normal scan: {output}");
    write_file(
        &root,
        "tools/boundary-check/snapshots/consumer-doc-fence-debt.toml",
        "[[documents]]\npath = 'docs/guide.md'\nunclassified = 0\n",
    );
    let (ok, output) = run_boundary_check(&root);
    let _ = std::fs::remove_dir_all(&root);
    assert!(!ok);
    assert!(
        output.contains("BC8006_CONSUMER_FENCE_DEBT_EXCEEDED"),
        "{output}"
    );
}
