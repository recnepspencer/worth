//! Bare `CHK24` inspection exempts burned monotonic allocation fields from
//! `model` while exposing them in `record_allocation_bytes` for full-invariance
//! checks (#463).
use super::{
    fields, map, original, request, rewritten, wider, ExecutionRequest, RelationalNativeCheckpoint,
    CHECKPOINT_MAGIC,
};

#[test]
fn bare_chk24_inspection_exempts_burned_allocations_from_model_and_exposes_allocation_bytes() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let (before, _) = original(&execution);
    let decoded = RelationalNativeCheckpoint::inspect_bytes(&execution, before.bytes()).unwrap();
    let envelope = fields(&before.bytes()[CHECKPOINT_MAGIC.len()..]);
    let manifest = fields(envelope["manifest"]);
    let identity = fields(manifest["record_identity"]);
    let roots = fields(identity["roots"]);

    for key in [
        "append_frontiers",
        "generation_high_water",
        "partition_high_water",
    ] {
        let equivalent = wider(roots[key])
            .unwrap_or_else(|| panic!("expected widenable CHK24 allocation root {key}"));
        let after_bytes = rewritten(before.bytes(), true, |root_fields| {
            root_fields.insert(key.into(), equivalent);
        });
        let after = RelationalNativeCheckpoint::inspect_bytes(&execution, &after_bytes).unwrap();
        assert_eq!(
            decoded.model, after.model,
            "burned monotonic allocation field {key} must not change model bytes"
        );
        assert_ne!(
            decoded.record_allocation_bytes, after.record_allocation_bytes,
            "full-invariance inspection must still observe {key}"
        );
    }

    let wider_pages = wider(identity["pages"]).expect("expected widenable CHK24 identity pages");
    let after_pages = rewritten(before.bytes(), false, |stored| {
        let value = map(&stored["record_identity"], |id_fields| {
            id_fields.insert("pages".into(), wider_pages);
        });
        stored.insert("record_identity".into(), value);
    });
    let after = RelationalNativeCheckpoint::inspect_bytes(&execution, &after_pages).unwrap();
    assert_eq!(
        decoded.model, after.model,
        "identity page roster must not change model bytes"
    );
    assert_ne!(
        decoded.record_allocation_bytes, after.record_allocation_bytes,
        "full-invariance inspection must still observe identity page roster changes"
    );
}
