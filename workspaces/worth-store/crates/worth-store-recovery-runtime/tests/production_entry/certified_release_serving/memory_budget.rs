//! A selected V2 source closure has one recovery-memory budget, not a fresh
//! full-sized allowance for each roster and media observation owner.

use std::{fs, io::ErrorKind, path::Path};

use worth_store::physical_runtime::{
    PhysicalRecoveryRejoinResidentDenial, RecoveryWalAllocationDenial,
};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome,
    PhysicalRecoverySourceDenial, WorthStoreRecovery,
};

use super::*;

#[test]
fn selected_release_source_memory_denies_before_effects_and_sufficient_twin_opens_serving() {
    let (world, first, _) = release_reopen::released_world(1);
    assert!(
        first.remaining_payload_records() > 0,
        "partial released head remains current"
    );
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xe1; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("actual release checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("v2-source-memory-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let current = selector(&root, RecordArtifactFile::CurrentRootSelector);
            let previous = selector(&root, RecordArtifactFile::PreviousRootSelector);
            let checkpoint = fs::read(root.join("families/checkpoint.current"))
                .expect("selected certified checkpoint");
            // Which owner denies first depends on fixture sizes; the claim is
            // that a closure below its measured peak blocks before effects.
            // Head-roster resident accounting is unit-tested in physics
            // `release_custody/head_v2/tests.rs`.
            let bounded = 512 << 10;
            let outcome = WorthStoreRecovery::recover(
                super::recovery_request::request_with_memory(&root, bounded),
            );
            let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
                panic!("bounded V2 source closure must block: {outcome:?}")
            };
            assert_eq!(blocked.recovery_effects(), 0);
            let evidence = blocked.evidence();
            let required = evidence
                .source_denials
                .iter()
                .find_map(|denial| match denial {
                    PhysicalRecoverySourceDenial::WalAdmissionAllocation {
                        cause:
                            RecoveryWalAllocationDenial::Backing {
                                cause:
                                    PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                                        required,
                                        admitted,
                                    },
                                ..
                            },
                    } if *admitted == bounded && *required > bounded => Some(*required),
                    _ => None,
                })
                .unwrap_or_else(|| {
                    panic!("the block must keep its typed recovery-memory cause: {evidence:?}")
                });
            // The same media recovers with more memory: a limit, not damage,
            // stated with both counts.
            let cause = blocked.cause();
            assert_eq!(cause.phase(), PhysicalRecoveryBlockKind::SourceAllocation);
            let limit = cause.limit().expect("recovery memory is a limit");
            assert_eq!(
                (limit.dimension(), limit.observed(), limit.admitted()),
                (
                    PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
                    required,
                    bounded
                )
            );
            assert_eq!(
                selector(&root, RecordArtifactFile::CurrentRootSelector),
                current
            );
            assert_eq!(
                selector(&root, RecordArtifactFile::PreviousRootSelector),
                previous
            );
            assert_eq!(
                fs::read(root.join("families/checkpoint.current")).unwrap(),
                checkpoint,
            );

            let outcome = WorthStoreRecovery::recover(super::request(&root));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("same selected media must recover with sufficient memory: {outcome:?}")
            };
            assert_eq!(handoff.core().recovery_effect_count(), 0);
            let planning_peak = handoff.planning_counters().peak_recovery_bytes();
            assert!(
                planning_peak > bounded,
                "the denied budget must sit below the measured recovery peak"
            );
            assert_eq!(
                selector(&root, RecordArtifactFile::CurrentRootSelector),
                current
            );
            assert_eq!(
                selector(&root, RecordArtifactFile::PreviousRootSelector),
                previous
            );
            assert_eq!(
                fs::read(root.join("families/checkpoint.current")).unwrap(),
                checkpoint,
            );
            // No Store-only WAL leg: one recovery-memory setting means C8's planning
            // peak always denies first; `wal_inventory/tests/resident_ceiling.rs` owns it.
            let seal = handoff
                .into_core()
                .into_checkpoint_custody()
                .expect("sufficient V2 source admission yields checkpoint seal");
            super::open_serving_with_seal(&root, seal);
        })
        .expect("bounded recovery worker");
    worker
        .join()
        .expect("bounded recovery worker did not panic");
}

fn selector(root: &Path, artifact: RecordArtifactFile) -> Option<Vec<u8>> {
    let path = root.join("families/records").join(artifact.file_name());
    match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => panic!("selected selector read failed: {error}"),
    }
}
