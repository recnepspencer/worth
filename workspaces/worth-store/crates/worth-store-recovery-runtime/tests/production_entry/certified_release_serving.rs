//! The certification pause path uses a genuinely selected C8 claim and the
//! same Store media rejoin used by production construction.

use super::*;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial;
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryStaticConfiguration, WorthStoreRecovery,
};

#[path = "certified_release_serving/tamper.rs"]
mod tamper;
use tamper::{
    alter_selected_checkpoint, alter_selected_control, alter_selected_file,
    alter_selected_routing_root,
};
#[path = "certified_release_serving/request.rs"]
mod recovery_request;
pub(super) use recovery_request::{request, request_with_memory, request_with_memory_and_format};
#[path = "certified_release_serving/plain_serving.rs"]
mod plain_serving;
pub(super) use plain_serving::assert_plain_serving_denied;
#[path = "certified_release_serving/empty_no_release.rs"]
mod empty_no_release;
pub(super) use empty_no_release::run_empty_no_release;
#[path = "certified_release_serving/head_diagnostics.rs"]
mod head_diagnostics;
#[path = "certified_release_serving/ledger_preparation.rs"]
mod ledger_preparation;
#[path = "certified_release_serving/memory_budget.rs"]
mod memory_budget;
#[path = "certified_release_serving/original_admission.rs"]
mod original_admission;
#[path = "certified_release_serving/pending_wal_rejoin.rs"]
mod pending_wal_rejoin;
#[path = "certified_release_serving/release_capacity_budget.rs"]
mod release_capacity_budget;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mutation {
    None,
    AfterClaimCheckpoint,
    BeforeFinalRejoinCheckpoint,
    BeforeFinalRejoinControl,
    AfterSealCheckpoint,
    AfterSealControl,
    AfterSealWal,
    AfterSealRoute,
}

pub(super) fn run(mutation: Mutation) {
    run_world(release_reopen::released_world(1024), mutation);
}

pub(super) fn run_tier_and_release() {
    run_world(release_reopen::released_tier_world(1024), Mutation::None);
}

fn run_world(
    (world, receipt, _): (
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
        worth_store::physical_runtime::BlobReclaimReceipt,
        worth_store_physical_format::PersistedRecordIdentity,
    ),
    mutation: Mutation,
) {
    assert_eq!(receipt.remaining_payload_records(), 0);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x7a; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("release checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "checkpoint after recovered Serving: {outcome:?}"
    );
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("certified-release-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || recover(root, mutation))
        .expect("recovery worker");
    worker.join().expect("recovery worker did not panic");
}

fn recover(root: PathBuf, mutation: Mutation) {
    let request = request(&root);
    let selected_checkpoint = root.join("families/checkpoint.current");
    let after_claim_reached = Arc::new(AtomicBool::new(false));
    let final_rejoin_reached = Arc::new(AtomicBool::new(false));
    let after_claim_flag = Arc::clone(&after_claim_reached);
    let final_rejoin_flag = Arc::clone(&final_rejoin_reached);
    let selected_descriptor = Arc::new(Mutex::new(None));
    let descriptor_for_claim = Arc::clone(&selected_descriptor);
    let descriptor_for_rejoin = Arc::clone(&selected_descriptor);
    let final_checkpoint = selected_checkpoint.clone();
    let final_root = root.clone();
    let outcome = WorthStoreRecovery::certification_recover_with_custody_pauses(
        request,
        move |descriptor| {
            after_claim_flag.store(true, Ordering::SeqCst);
            *descriptor_for_claim.lock().unwrap() = Some(descriptor);
            if mutation == Mutation::AfterClaimCheckpoint {
                alter_selected_checkpoint(&selected_checkpoint);
            }
        },
        move || {
            final_rejoin_flag.store(true, Ordering::SeqCst);
            if mutation == Mutation::BeforeFinalRejoinCheckpoint {
                alter_selected_checkpoint(&final_checkpoint);
            } else if mutation == Mutation::BeforeFinalRejoinControl {
                alter_selected_control(
                    &final_root,
                    descriptor_for_rejoin
                        .lock()
                        .unwrap()
                        .expect("selected descriptor route"),
                );
            }
        },
    );
    assert!(
        after_claim_reached.load(Ordering::SeqCst),
        "C8 never minted the claim"
    );
    if mutation != Mutation::None
        && mutation != Mutation::AfterSealCheckpoint
        && mutation != Mutation::AfterSealControl
        && mutation != Mutation::AfterSealWal
        && mutation != Mutation::AfterSealRoute
    {
        if mutation == Mutation::BeforeFinalRejoinCheckpoint {
            assert!(
                final_rejoin_reached.load(Ordering::SeqCst),
                "Store initial join was not reached"
            );
        }
        let PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) = outcome else {
            panic!("altered selected media must deny at Store handoff: {outcome:?}")
        };
        assert_eq!(
            indeterminate.handoff_failure(),
            Some(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch),
        );
        return;
    }
    assert!(
        final_rejoin_reached.load(Ordering::SeqCst),
        "Store final reread was not reached"
    );
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("genuine selected custody failed Store rejoin: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("selected C8 custody seal");
    if mutation == Mutation::AfterSealCheckpoint {
        alter_selected_checkpoint(&root.join("families/checkpoint.current"));
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation == Mutation::AfterSealControl {
        alter_selected_control(
            &root,
            selected_descriptor
                .lock()
                .unwrap()
                .expect("selected descriptor route"),
        );
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation == Mutation::AfterSealWal {
        alter_selected_file(&root.join("families/wal"), |name| name.ends_with(".wal"));
        open_serving_inner(&root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
    } else if mutation == Mutation::AfterSealRoute {
        alter_selected_routing_root(&root);
        open_serving_inner(
            &root,
            Some(seal),
            Some(worth_store::physical_runtime::RecordBootstrapDenial::CurrentRootDamaged),
            true,
        );
    } else {
        open_serving_with_seal(&root, seal);
    }
}

pub(super) fn open_serving_with_seal(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
) {
    let _ = open_serving_inner(root, Some(seal), None, true);
}

pub(super) fn open_serving_with_seal_without_checkpoint(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
) {
    admit_serving_with_seal(root, seal).close();
}

pub(super) fn admit_serving_with_seal(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_serving_inner(root, Some(seal), None, false)
        .expect("verified C8 custody must produce Serving")
}

pub(super) fn admit_serving_with_seal_and_format(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_serving_inner_with_format(
        root,
        Some(seal),
        None,
        false,
        NonZeroU64::new(16 << 20).unwrap(),
        format,
    )
    .expect("verified C8 custody must produce format-matched Serving")
}

pub(super) fn admit_serving_with_seal_and_wal_segment_bytes(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    wal_segment_bytes: NonZeroU64,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_serving_inner_with_wal_segment_bytes(root, Some(seal), None, false, wal_segment_bytes)
        .expect("verified C8 custody must produce Serving")
}

pub(super) fn open_serving_with_seal_expect_mismatch(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
) {
    let _ = open_serving_inner(root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
}

fn open_serving_inner(
    root: &Path,
    seal: Option<worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody>,
    expected_denial: Option<worth_store::physical_runtime::RecordBootstrapDenial>,
    require_checkpoint: bool,
) -> Option<worth_store::physical_runtime::ServingPhysicalRuntime> {
    open_serving_inner_with_wal_segment_bytes(
        root,
        seal,
        expected_denial,
        require_checkpoint,
        NonZeroU64::new(16 << 20).unwrap(),
    )
}

fn open_serving_inner_with_wal_segment_bytes(
    root: &Path,
    seal: Option<worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody>,
    expected_denial: Option<worth_store::physical_runtime::RecordBootstrapDenial>,
    require_checkpoint: bool,
    wal_segment_bytes: NonZeroU64,
) -> Option<worth_store::physical_runtime::ServingPhysicalRuntime> {
    use worth_store::physical_runtime::{
        AdmittedPhysicalRecordFormat, PhysicalRecordFormatDeclaration,
    };
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    open_serving_inner_with_format(
        root,
        seal,
        expected_denial,
        require_checkpoint,
        wal_segment_bytes,
        format,
    )
}

fn open_serving_inner_with_format(
    root: &Path,
    seal: Option<worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody>,
    expected_denial: Option<worth_store::physical_runtime::RecordBootstrapDenial>,
    require_checkpoint: bool,
    wal_segment_bytes: NonZeroU64,
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
) -> Option<worth_store::physical_runtime::ServingPhysicalRuntime> {
    use std::num::NonZeroU32;
    use worth_store::physical_runtime::{
        CheckpointMemoryLimit, FilesystemAccessPosture, FilesystemMediaAdmission, GroupCommitDelay,
        GroupCommitLimit, IdempotencyRetentionGenerations, LiveIdempotencyBindingLimit,
        PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration,
        PhysicalIdempotencyPolicy, PhysicalRecordAccessPolicy, PhysicalRecordOpen,
        PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, RetainedWalTailLimit,
        WalSegmentByteLimit, WalSegmentInventoryLimit,
    };

    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("serving media after C8 must admit")
    };
    let TransitionOutcome::Success(durability) = PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(wal_segment_bytes),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1024).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 << 20).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 << 20).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    else {
        panic!("serving durability after C8 must admit")
    };
    let open = PhysicalRecordOpen::new(format, access, durability);
    let open = match seal {
        Some(seal) => open.with_recovered_checkpoint_custody(seal),
        None => open,
    };
    let opened = media.open_record_store(open).into_raw();
    if let Some(expected_denial) = expected_denial {
        let TransitionOutcome::Denied(denial) = opened else {
            panic!("post-seal selected-media substitution must deny at Serving open")
        };
        assert_eq!(denial.reason(), expected_denial);
        return None;
    }
    let serving = match opened {
        TransitionOutcome::Success(serving) => serving,
        TransitionOutcome::Denied(denial) => {
            panic!("verified C8 custody Serving denial: {:?}", denial.reason())
        }
        TransitionOutcome::Stale(stale) => {
            panic!("verified C8 custody Serving stale: {:?}", stale.reason())
        }
        TransitionOutcome::RebindRequired(rebind) => {
            panic!("verified C8 custody Serving rebind: {:?}", rebind.reason())
        }
        TransitionOutcome::Deferred(deferred) => match deferred {},
        TransitionOutcome::Failed(failure) => {
            panic!(
                "verified C8 custody Serving inspection/failure: {:?}",
                failure.cause()
            )
        }
    };
    if !require_checkpoint {
        return Some(serving);
    }
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x7b; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(checkpoint).into_raw()
    else {
        panic!("checkpoint on recovered Serving must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "checkpoint after recovered Serving: {outcome:?}"
    );
    serving.close();
    None
}
