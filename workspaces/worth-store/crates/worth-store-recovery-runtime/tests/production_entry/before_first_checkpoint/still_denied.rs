//! What the generation-zero basis must never admit: a store whose media
//! shows something only a checkpoint could have made, or whose checkpoint is
//! present but unreadable.

use worth_store::physical_runtime::RecordBootstrapDenial;
use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, RecordArtifactFile,
};
use worth_store_recovery_runtime::{
    HistoricalDropAdmissionStage, PhysicalRecoveryBlockKind, PhysicalRecoveryPageAdmissionDenial,
    PhysicalRecoveryPlanningDenial,
};

use super::*;

/// Recovery must block in `phase`, with the planning denial `denial` admits,
/// never open the generation-zero basis.
fn assert_blocked(
    root: &Path,
    stage: &str,
    phase: PhysicalRecoveryBlockKind,
    denial: fn(Option<&PhysicalRecoveryPlanningDenial>) -> bool,
) {
    match WorthStoreRecovery::recover(certified_release_serving::request(root)) {
        PhysicalRecoveryOutcome::Blocked(block) => assert!(
            block.cause().damage() == Some(phase)
                && denial(block.evidence().planning_denial.as_ref()),
            "{stage} blocked elsewhere: kind={:?}; cause={:?}",
            block.cause(),
            block.evidence().planning_denial
        ),
        PhysicalRecoveryOutcome::Recovered(_) => panic!("{stage} opened without a checkpoint"),
        _ => panic!("{stage} neither blocked nor recovered"),
    }
}

#[test]
fn a_present_but_rejected_checkpoint_never_becomes_generation_zero() {
    for (case, bytes) in [
        ("garbage", b"not a checkpoint stream".to_vec()),
        ("empty", Vec::new()),
    ] {
        let world = KilledWorld::launch(FirstWrite::Record);
        fs::write(world.root.join("families/checkpoint.current"), bytes).unwrap();
        // Discovery rejects the stream itself; damage never reads as absence.
        assert_blocked(
            &world.root,
            case,
            PhysicalRecoveryBlockKind::Checkpoint,
            |denial| denial.is_none(),
        );
    }
}

#[test]
fn a_retained_released_drop_without_a_checkpoint_stays_denied() {
    let (world, _, _) = crate::release_reopen::released_world(1);
    let root = world.retained_root();
    drop(world);
    fs::remove_file(root.path().join("families/checkpoint.current")).unwrap();
    // The ordered-history walk start denies the drop without a checkpoint.
    assert_blocked(
        root.path(),
        "a released drop",
        PhysicalRecoveryBlockKind::PageAdmission,
        |denial| {
            matches!(
                denial,
                Some(PhysicalRecoveryPlanningDenial::Page(
                    PhysicalRecoveryPageAdmissionDenial::HistoricalDrop {
                        stage: HistoricalDropAdmissionStage::OrderedHistory,
                        ..
                    }
                ))
            )
        },
    );
}

#[test]
fn a_free_space_tier_start_without_a_checkpoint_stays_denied() {
    let world = KilledWorld::launch(FirstWrite::Record);
    start_a_tier_epoch_in_the_free_space_header(&world.root);
    // A free-space tier start needs the root anchor only a checkpoint makes.
    assert_blocked(
        &world.root,
        "a free-space tier epoch start",
        PhysicalRecoveryBlockKind::PageAdmission,
        |denial| {
            matches!(
                denial,
                Some(PhysicalRecoveryPlanningDenial::Page(
                    PhysicalRecoveryPageAdmissionDenial::InvalidManifest { .. }
                ))
            )
        },
    );
}

/// Rewrites the selected root's free-space header to start a tier epoch, and
/// rebinds the root to the rewritten header so only the epoch is wrong.
fn start_a_tier_epoch_in_the_free_space_header(root: &Path) {
    let records = root.join("families/records");
    let selector = fs::read(records.join(RecordArtifactFile::CurrentRootSelector.file_name()))
        .expect("current root selector");
    let generation = DurableRootSelector::decode(&selector)
        .unwrap()
        .root_generation();
    let root_path = records
        .join("roots")
        .join(RecordArtifactFile::RootManifest { generation }.file_name());
    let (manifest, format) =
        DurablePhysicalRootManifest::decode(&fs::read(&root_path).unwrap(), u16::MAX).unwrap();
    assert!(manifest.tier_epoch_anchor().is_none());
    let header_path = records
        .join("free-space")
        .join(RecordArtifactFile::FreeSpaceManifest { generation }.file_name());
    let (header, _) =
        DurableFreeSpaceManifestHeader::decode(&fs::read(&header_path).unwrap(), u16::MAX).unwrap();
    assert!(header.tier_epoch_start().is_none());
    let started = DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        header.generation(),
        header.tree_identity(),
        header.node_capacity(),
        header.segment_page_capacity(),
        header.entry_count(),
        header.next_segment(),
        header.next_page(),
        header.next_extent(),
        header.next_arena(),
        Some(header.next_arena()),
        header.arena_capacity(),
        header.arena_alignment(),
        header.next_block(),
        header.root(),
    )
    .expect("a structurally valid tier epoch start")
    .encode(format);
    fs::write(&header_path, &started).unwrap();
    let rebound = DurablePhysicalRootManifest::builder(
        manifest.generation(),
        manifest.tree_identity(),
        manifest.node_capacity(),
        durable_artifact_checksum(&started),
    )
    .record_count(manifest.record_count())
    .next_block(manifest.next_block())
    .next_segment_block(manifest.next_segment_block())
    .routing_root(manifest.routing_root())
    .segment_root(manifest.segment_root())
    .free_space_root(manifest.free_space_root())
    .last_inline_record(manifest.last_inline_record())
    .last_inline_segment(manifest.last_inline_segment())
    .admit()
    .unwrap();
    fs::write(&root_path, rebound.encode(format)).unwrap();
}

/// The generation-zero seal holds only while `checkpoint.current` stays absent:
/// a stream that appears between recovery and Serving is not this basis,
/// whether its bytes overrun the absent ceiling or it is empty.
#[test]
fn a_checkpoint_that_appears_after_generation_zero_recovery_denies_serving() {
    let overrun: certified_release_serving::ExpectedServingDenial =
        |denial| matches!(denial, RecordBootstrapDenial::RecoveredCheckpointRead(_));
    let mismatch: certified_release_serving::ExpectedServingDenial =
        |denial| *denial == RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch;
    for (bytes, expected) in [
        (b"a stream that appeared after recovery".as_slice(), overrun),
        (b"".as_slice(), mismatch),
    ] {
        let world = KilledWorld::launch(FirstWrite::Record);
        let seal = world
            .recover("generation-zero recovery")
            .into_core()
            .into_checkpoint_custody()
            .expect("generation-zero custody seal");
        fs::write(world.root.join("families/checkpoint.current"), bytes).unwrap();
        certified_release_serving::open_serving_with_seal_expect_denial(
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
    let world = KilledWorld::launch(FirstWrite::Record);
    let serving = world.serve("first recovery");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xd4; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("the first checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();
    fs::remove_file(world.root.join("families/checkpoint.current")).unwrap();
    let serving = world.serve("recovery with the checkpoint deleted");
    world.assert_reads_back(&serving);
    serving.close();
}
