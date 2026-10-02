//! Two completed releases rejoin without an ordinary tail or new checkpoint,
//! then retain exact per-object progress across checkpoint and fresh recovery.

use super::*;
use worth_store_physical_format::{DurablePhysicalRootManifest, ReleaseCustodyHeadEntryV1};
use worth_store_physical_integrity::{walk_release_custody_head, ReleaseCustodyHeadWalkLimitsV1};

#[test]
fn two_completed_releases_without_ordinary_tail_serve_checkpoint_and_reopen_exact_heads() {
    std::thread::Builder::new()
        .name("completed-release-tip".into())
        .stack_size(16 << 20)
        .spawn(|| {
            let world = pending_wal_world::first();
            let checkpoint = fs::read(world.root().join("families/checkpoint.current")).unwrap();
            world.kill_distinct_release_before_checkpoint();
            let outcome = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("real pending second release must recover")
            };
            assert!(handoff.core().recovery_effect_count() > 0);
            let seal = handoff.into_core().into_checkpoint_custody().unwrap();
            let serving = certified_release_serving::admit_serving_with_seal(world.root(), seal);
            serving.close();
            let selected = selected_generation(world.root());
            let expected_heads = selected_root_heads(world.root());
            assert_eq!(expected_heads.len(), 2);
            assert_ne!(expected_heads[0].key(), expected_heads[1].key());
            assert!(expected_heads.iter().all(|head| head.cumulative_dropped() == 1));
            assert_eq!(fs::read(world.root().join("families/checkpoint.current")).unwrap(), checkpoint,
                "the pending-release recovery must preserve the original NoRelease checkpoint");
            let outcome = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
            let handoff = match outcome {
                PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
                PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
                    "completed Store rejoin failed: handoff={:?}; reopen={:?}; effects={}",
                    failure.handoff_failure(), failure.reopen_failure(), failure.recovery_effects()),
                PhysicalRecoveryOutcome::Blocked(block) => panic!(
                    "completed C8 admission blocked: kind={:?}; artifact={:?}; cause={:?}; effects={}",
                    block.kind, block.evidence().artifact, block.evidence().planning_denial,
                    block.recovery_effects()),
                other => panic!("completed history must independently rejoin Store: {other:?}"),
            };
            assert_eq!(handoff.core().recovery_effect_count(), 0,
                "completed releases must not be repeated");
            assert_eq!(selected_generation(world.root()), selected);
            assert_eq!(fs::read(world.root().join("families/checkpoint.current")).unwrap(), checkpoint);
            let seal = handoff.into_core().into_checkpoint_custody()
                .expect("independently verified completed history must issue a seal");
            let allocations = seal.certification_residency_allocations();
            let recovery = worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(
                worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
            assert!(allocations.snapshot().for_dimension(recovery).active_units() > 0,
                "completed-history evidence must retain native backing in the seal");
            let serving = certified_release_serving::admit_serving_with_seal(world.root(), seal);
            checkpoint_completed_history(&serving);
            serving.close();
            let disposed = allocations.snapshot().for_dimension(recovery);
            assert_eq!(disposed.active_units(), 0);
            assert_eq!(disposed.admitted_units(), disposed.released_units());
            let (_, accumulator) = release_reopen::selected_release_certificates_from_bytes(
                &fs::read(world.root().join("families/checkpoint.current")).unwrap());
            assert_eq!(release_reopen::selected_head_oracle::selected_heads(world.root(), accumulator),
                expected_heads, "checkpoint must retain both objects' exact progress");
            let PhysicalRecoveryOutcome::Recovered(handoff) = WorthStoreRecovery::recover(
                certified_release_serving::request(world.root())) else {
                panic!("completed-history checkpoint must freshly recover")
            };
            assert_eq!(handoff.core().recovery_effect_count(), 0);
            let seal = handoff.into_core().into_checkpoint_custody().unwrap();
            certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
            assert_eq!(selected_root_heads(world.root()), expected_heads,
                "fresh Serving must preserve exact A/B custody");
        })
        .unwrap().join().expect("completed release tip worker");
}

fn checkpoint_completed_history(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xc9; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("completed-history Serving checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "completed-history checkpoint must complete: {outcome:?}"
    );
}

fn selected_root_heads(root: &Path) -> Vec<ReleaseCustodyHeadEntryV1> {
    let records = root.join("families/records/roots");
    let bytes = fs::read(
        records.join(
            RecordArtifactFile::RootManifest {
                generation: selected_generation(root),
            }
            .file_name(),
        ),
    )
    .unwrap();
    let (manifest, format) = DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
    let limits = ReleaseCustodyHeadWalkLimitsV1::new(
        32,
        2,
        32 * u64::from(format.page_size().bytes()),
        1 << 20,
        16,
    )
    .unwrap();
    let mut entries = Vec::new();
    let walk = walk_release_custody_head(
        &manifest,
        format,
        limits,
        |reference, remaining| {
            let bytes = fs::read(
                records.join(
                    RecordArtifactFile::ReleaseCustodyHeadBlock {
                        generation: reference.generation(),
                        block: reference.block(),
                    }
                    .file_name(),
                ),
            )?;
            assert!(bytes.len() as u64 <= remaining);
            Ok::<_, std::io::Error>(bytes)
        },
        |entry| {
            entries.push(entry);
            Ok::<_, ()>(())
        },
    )
    .expect("independent selected-media head walk");
    assert_eq!(walk.entry_count(), 2);
    entries
}
