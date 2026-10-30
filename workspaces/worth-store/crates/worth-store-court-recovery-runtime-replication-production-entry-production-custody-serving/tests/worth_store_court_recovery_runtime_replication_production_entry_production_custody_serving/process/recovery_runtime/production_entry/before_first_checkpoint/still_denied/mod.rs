//! What the generation-zero basis must never admit: a store whose media
//! shows something only a checkpoint could have made, or whose checkpoint is
//! present but unreadable.

#[allow(unused_imports)]
use crate::production_entry::before_first_checkpoint::still_denied::*;
use worth_store::physical_runtime::RecordBootstrapDenial;
use {
    worth_store_physical_format::durable_artifact_checksum,
    worth_store_physical_format::DurableFreeSpaceManifestHeader,
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::DurableRootSelector,
    worth_store_physical_format::RecordArtifactFile,
};
use {
    worth_store_recovery_runtime::HistoricalDropAdmissionStage,
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryPageAdmissionDenial,
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial,
};

use {
    crate::production_entry::before_first_checkpoint::FirstWrite,
    crate::production_entry::before_first_checkpoint::KilledWorld, std::fs, std::path::Path,
    worth_proof::TransitionOutcome, worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen,
};

#[test]
fn a_present_but_rejected_checkpoint_never_becomes_generation_zero() {
    for (case, bytes) in [
        ("garbage", b"not a checkpoint stream".to_vec()),
        ("empty", Vec::new()),
    ] {
        let world = crate::production_entry::before_first_checkpoint::KilledWorld::launch(
            crate::production_entry::before_first_checkpoint::FirstWrite::Record,
        );
        std::fs::write(world.root.join("families/checkpoint.current"), bytes).unwrap();
        // Discovery rejects the stream itself; damage never reads as absence.
        assert_blocked(
            &world.root,
            case,
            worth_store_recovery_runtime::PhysicalRecoveryBlockKind::Checkpoint,
            |denial| denial.is_none(),
        );
    }
}

#[test]
fn a_retained_released_drop_without_a_checkpoint_stays_denied() {
    let (world, _, _) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1);
    let root = world.retained_root();
    drop(world);
    std::fs::remove_file(root.path().join("families/checkpoint.current")).unwrap();
    // The ordered-history walk start denies the drop without a checkpoint.
    assert_blocked(
        root.path(),
        "a released drop",
        worth_store_recovery_runtime::PhysicalRecoveryBlockKind::PageAdmission,
        |denial| {
            matches!(
                denial,
                Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::Page(
                    worth_store_recovery_runtime::PhysicalRecoveryPageAdmissionDenial::HistoricalDrop {
                        stage: worth_store_recovery_runtime::HistoricalDropAdmissionStage::OrderedHistory,
                        ..
                    }
                ))
            )
        },
    );
}

#[test]
fn a_free_space_tier_start_without_a_checkpoint_stays_denied() {
    let world = crate::production_entry::before_first_checkpoint::KilledWorld::launch(
        crate::production_entry::before_first_checkpoint::FirstWrite::Record,
    );
    start_a_tier_epoch_in_the_free_space_header(&world.root);
    // A free-space tier start needs the root anchor only a checkpoint makes.
    assert_blocked(
        &world.root,
        "a free-space tier epoch start",
        worth_store_recovery_runtime::PhysicalRecoveryBlockKind::PageAdmission,
        |denial| {
            matches!(
                denial,
                Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::Page(
                    worth_store_recovery_runtime::PhysicalRecoveryPageAdmissionDenial::InvalidManifest { .. }
                ))
            )
        },
    );
}

/// The generation-zero seal holds only while `checkpoint.current` stays absent:
/// a stream that appears between recovery and Serving is not this basis,
/// whether its bytes overrun the absent ceiling or it is empty.
#[test]
fn a_checkpoint_that_appears_after_generation_zero_recovery_denies_serving() {
    let overrun: worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::ExpectedServingDenial =
        |denial| matches!(denial, worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointRead(_));
    let mismatch: worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::ExpectedServingDenial =
        |denial| *denial == worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch;
    for (bytes, expected) in [
        (b"a stream that appeared after recovery".as_slice(), overrun),
        (b"".as_slice(), mismatch),
    ] {
        let world = crate::production_entry::before_first_checkpoint::KilledWorld::launch(
            crate::production_entry::before_first_checkpoint::FirstWrite::Record,
        );
        let seal = world
            .recover("generation-zero recovery")
            .into_core()
            .certification_release_media_into_checkpoint_custody()
            .expect("generation-zero custody seal");
        std::fs::write(world.root.join("families/checkpoint.current"), bytes).unwrap();
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_expect_denial(
            &world.root,
            seal,
            expected,
        );
    }
}

/// A deleted `checkpoint.current` over a store whose plain checkpoint left no
/// other trace, with the whole WAL retained, opens as generation zero exactly
/// as the store did before that checkpoint (parity with ordinary open).
#[test]
fn a_deleted_checkpoint_with_the_whole_wal_retained_opens_as_generation_zero() {
    let world = crate::production_entry::before_first_checkpoint::KilledWorld::launch(
        crate::production_entry::before_first_checkpoint::FirstWrite::Record,
    );
    let serving = world.serve("first recovery");
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xd4; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("the first checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();
    std::fs::remove_file(world.root.join("families/checkpoint.current")).unwrap();
    let serving = world.serve("recovery with the checkpoint deleted");
    world.assert_reads_back(&serving);
    serving.close();
}
