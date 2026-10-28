//! One genuine partial release populates the custody used by the policy proof.

use super::*;

pub(super) fn checkpointed_partial_release_world() -> PhysicalResidencyStoreWorld {
    let (world, release, _) = release_reopen::released_world(1);
    assert_eq!(release.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(release.dropped_records().len(), 1);
    assert!(release.remaining_payload_records() > 0);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xb7; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("partial-release checkpoint must admit")
    };
    assert!(
        matches!(handle.wait(), PhysicalCheckpointOutcome::Completed(_)),
        "partial-release custody checkpoint must complete"
    );
    world
}
