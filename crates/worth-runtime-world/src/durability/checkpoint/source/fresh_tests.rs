//! Empty protected-cut meaning only; no Native or physical authority is fabricated.
use super::WorldCheckpointReadView;
use crate::durability::WorldReadmissionDenial;
use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

#[test]
fn fresh_world_source_requires_no_selected_checkpoint_and_no_journal_records() {
    assert!(
        WorldCheckpointReadView::from_protected_home_source(None, [0; 3])
            .unwrap()
            .require_fresh_empty()
            .is_ok()
    );
    for heads in [[1, 0, 0], [0, 1, 0], [0, 0, 1]] {
        assert!(matches!(
            WorldCheckpointReadView::from_protected_home_source(None, heads)
                .unwrap()
                .require_fresh_empty(),
            Err(WorldReadmissionDenial::FreshWorldSourceNotEmpty)
        ));
    }
    // A real selected slot is disallowed even when its described byte range is empty.
    let bytes =
        ExecutionImmutableBytes::from_external_bytes(Arc::new(Vec::new().into_boxed_slice()));
    assert!(matches!(
        WorldCheckpointReadView::from_protected_home_source(Some((bytes, 0..0)), [0; 3])
            .unwrap()
            .require_fresh_empty(),
        Err(WorldReadmissionDenial::FreshWorldSourceNotEmpty)
    ));
}
