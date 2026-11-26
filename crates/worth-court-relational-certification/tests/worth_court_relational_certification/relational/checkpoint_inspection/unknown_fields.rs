//! Unknown root and CHK24 allocator-root fields refuse inspection.
use super::{map, original, request, rewritten, ExecutionRequest, RelationalNativeCheckpoint};

#[test]
fn inspection_refuses_unknown_root_allocator_and_row_fields() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let (before, _) = original(&execution);
    for allocator in [false, true] {
        let after = rewritten(before.bytes(), allocator, |fields| {
            fields.insert("future_field".into(), vec![0xc0]);
        });
        assert!(RelationalNativeCheckpoint::inspect_bytes(&execution, &after).is_err());
    }
    for key in ["append_frontiers", "generation_high_water"] {
        let after = rewritten(before.bytes(), true, |fields| {
            let root = map(&fields[key], |root| {
                root.insert("future_field".into(), vec![0xc3]);
            });
            fields.insert(key.into(), root);
        });
        assert!(
            RelationalNativeCheckpoint::inspect_bytes(&execution, &after).is_err(),
            "unknown field in {key} root"
        );
    }
}
