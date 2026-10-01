use super::*;
use worth_store::physical_runtime::{
    PhysicalCheckpointCaptureFailureKind, PhysicalCheckpointStartFailure,
};

pub(crate) fn assert_plain_serving_checkpoint_unavailable(root: &Path) {
    let checkpoint_path = root.join("families/checkpoint.current");
    let before = std::fs::read(&checkpoint_path).expect("selected checkpoint before denied start");
    let serving = open_serving_inner(root, None, None, false)
        .expect("ordinary read-only Serving open after failed C8 handoff");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xc1; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    assert!(matches!(
        serving.checkpoints().start(request).into_raw(),
        TransitionOutcome::Failed(PhysicalCheckpointStartFailure::Capture(
            PhysicalCheckpointCaptureFailureKind::CheckpointCustodyUnavailable
        ))
    ));
    serving.close();
    assert_eq!(
        std::fs::read(checkpoint_path).expect("selected checkpoint after denied start"),
        before,
        "plain Serving checkpoint denial must leave selected checkpoint unchanged"
    );
}
