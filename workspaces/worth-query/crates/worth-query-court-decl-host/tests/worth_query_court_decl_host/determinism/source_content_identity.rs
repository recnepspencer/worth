use crate::aspect_patterns::source_subgraph::courts::fingerprint;
#[test]
fn canonical_source_content_fingerprint_includes_member_content_and_ignores_runtime_placement() {
    let original = fingerprint(11);
    let placed_elsewhere = fingerprint(11);
    assert_eq!(placed_elsewhere.0, original.0);
    assert_ne!(
        placed_elsewhere.1, original.1,
        "native provenance stays distinct"
    );
    assert_ne!(
        fingerprint(15).0,
        original.0,
        "a root-only fingerprint would miss this edit"
    );
}
