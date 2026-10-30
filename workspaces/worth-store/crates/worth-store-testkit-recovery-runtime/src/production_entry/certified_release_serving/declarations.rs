//! The certification pause path uses a genuinely selected C8 claim and the
//! same Store media rejoin used by production construction.

pub use std::path::Path;
pub use worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial;
pub use {
    crate::production_entry::release_reopen, std::num::NonZeroU64, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
pub use {
    std::sync::atomic::AtomicBool, std::sync::atomic::Ordering, std::sync::Arc, std::sync::Mutex,
};
pub use {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority,
    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub use {
    crate::certified_release_serving::tamper::alter_selected_checkpoint,
    crate::certified_release_serving::tamper::alter_selected_control,
    crate::certified_release_serving::tamper::alter_selected_file,
    crate::certified_release_serving::tamper::alter_selected_routing_root,
};

pub use {
    crate::production_entry::certified_release_serving::recovery_request::request,
    crate::production_entry::certified_release_serving::recovery_request::request_for_long_ingest,
    crate::production_entry::certified_release_serving::recovery_request::request_for_long_ingest_with_manifest_entries,
    crate::production_entry::certified_release_serving::recovery_request::request_narrowing,
    crate::production_entry::certified_release_serving::recovery_request::request_with_configuration,
    crate::production_entry::certified_release_serving::recovery_request::request_with_manifest_entries,
    crate::production_entry::certified_release_serving::recovery_request::request_with_memory,
    crate::production_entry::certified_release_serving::recovery_request::request_with_memory_and_format,
    crate::production_entry::certified_release_serving::recovery_request::ADMITTED_BYTES,
};

pub use crate::production_entry::certified_release_serving::plain_serving::assert_plain_serving_denied;

pub use crate::production_entry::certified_release_serving::empty_no_release::run_empty_no_release;

pub use crate::production_entry::certified_release_serving::recovery::recover;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mutation {
    None,
    AfterClaimCheckpoint,
    BeforeFinalRejoinCheckpoint,
    BeforeFinalRejoinControl,
    AfterSealCheckpoint,
    AfterSealControl,
    AfterSealWal,
    AfterSealRoute,
}

pub fn run(mutation: Mutation) {
    run_world(
        crate::production_entry::release_reopen::released_world(1024),
        mutation,
    );
}

pub fn run_tier_and_release() {
    run_world(
        crate::production_entry::release_reopen::released_tier_world(1024),
        crate::production_entry::certified_release_serving::Mutation::None,
    );
}

#[expect(
    clippy::disallowed_methods,
    reason = "Provides the fixture stack for a real production recovery and release journey."
)]
pub fn run_world(
    (world, receipt, _): (
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
        worth_store::physical_runtime::BlobReclaimReceipt,
        worth_store_physical_format::PersistedRecordIdentity,
    ),
    mutation: Mutation,
) {
    assert_eq!(receipt.remaining_payload_records(), 0);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x7a; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("release checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
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

pub fn open_serving_with_seal(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
) {
    let _ = open_serving_inner(root, Some(seal), None, true);
}

pub fn open_serving_with_seal_without_checkpoint(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
) {
    admit_serving_with_seal(root, seal).close();
}

pub fn admit_serving_with_seal(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_serving_inner(root, Some(seal), None, false)
        .expect("verified C8 custody must produce Serving")
}

pub fn admit_serving_with_seal_and_format(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_serving_inner_with_format(
        root,
        Some(seal),
        None,
        false,
        std::num::NonZeroU64::new(16 << 20).unwrap(),
        format,
        None,
    )
    .expect("verified C8 custody must produce format-matched Serving")
}

pub fn admit_serving_with_seal_format_and_policy(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    policy: worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_serving_inner_with_format(
        root,
        Some(seal),
        None,
        false,
        std::num::NonZeroU64::new(16 << 20).unwrap(),
        format,
        Some(policy),
    )
    .expect("verified C8 custody must produce policy-matched Serving")
}

pub fn admit_serving_with_seal_and_wal_segment_bytes(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    wal_segment_bytes: NonZeroU64,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_serving_inner_with_wal_segment_bytes(root, Some(seal), None, false, wal_segment_bytes)
        .expect("verified C8 custody must produce Serving")
}

pub fn open_serving_with_seal_expect_mismatch(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
) {
    let _ = open_serving_inner(root, Some(seal), Some(worth_store::physical_runtime::RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch), true);
}

/// Requires Serving open under `seal` to deny with a reason `expected` admits.
pub fn open_serving_with_seal_expect_denial(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    expected: ExpectedServingDenial,
) {
    let _ = open_serving_inner_with_wal_segment_bytes(
        root,
        Some(seal),
        Some(&expected),
        true,
        std::num::NonZeroU64::new(16 << 20).unwrap(),
    );
}

pub type ExpectedServingDenial = fn(&worth_store::physical_runtime::RecordBootstrapDenial) -> bool;

/// Which Serving denial an open must end in, if any.
pub type ServingDenialCheck<'check> =
    &'check dyn Fn(&worth_store::physical_runtime::RecordBootstrapDenial) -> bool;

pub fn open_serving_inner(
    root: &Path,
    seal: Option<worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody>,
    expected_denial: Option<worth_store::physical_runtime::RecordBootstrapDenial>,
    require_checkpoint: bool,
) -> Option<worth_store::physical_runtime::ServingPhysicalRuntime> {
    let equals = expected_denial.map(|expected| {
        move |denial: &worth_store::physical_runtime::RecordBootstrapDenial| *denial == expected
    });
    open_serving_inner_with_wal_segment_bytes(
        root,
        seal,
        equals.as_ref().map(|check| check as ServingDenialCheck<'_>),
        require_checkpoint,
        std::num::NonZeroU64::new(16 << 20).unwrap(),
    )
}

pub fn open_serving_inner_with_wal_segment_bytes(
    root: &Path,
    seal: Option<worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody>,
    expected_denial: Option<ServingDenialCheck<'_>>,
    require_checkpoint: bool,
    wal_segment_bytes: NonZeroU64,
) -> Option<worth_store::physical_runtime::ServingPhysicalRuntime> {
    use {
        worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
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
        None,
    )
}

pub fn open_serving_inner_with_format(
    root: &Path,
    seal: Option<worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody>,
    expected_denial: Option<ServingDenialCheck<'_>>,
    require_checkpoint: bool,
    wal_segment_bytes: NonZeroU64,
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    policy: Option<worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy>,
) -> Option<worth_store::physical_runtime::ServingPhysicalRuntime> {
    use {
        worth_store::physical_runtime::FilesystemAccessPosture,
        worth_store::physical_runtime::FilesystemMediaAdmission,
        worth_store::physical_runtime::PhysicalRecordAccessPolicy,
        worth_store::physical_runtime::PhysicalRecordOpen,
        worth_store::physical_runtime::PhysicalRuntimeAdmission,
        worth_store::physical_runtime::PhysicalStore,
    };

    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let worth_proof::TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("serving media after C8 must admit")
    };
    let worth_proof::TransitionOutcome::Success(durability) = serving_durability(wal_segment_bytes)
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    else {
        panic!("serving durability after C8 must admit")
    };
    let open = PhysicalRecordOpen::new(format, access, durability).with_journal_registry(
        worth_store::physical_runtime::OrderedJournalRegistry::declare(
            2,
            &[
                worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS,
                worth_store::physical_runtime::RELATIONAL_BRANCH_TRANSITIONS,
            ],
            &["home", "relational.checkpoint"],
        )
        .unwrap(),
    );
    let open = match policy {
        Some(policy) => open.with_residency_policy(policy),
        None => open,
    };
    let open = match seal {
        Some(seal) => open.with_recovered_checkpoint_custody(seal),
        None => open,
    };
    let opened = media.open_record_store(open).into_raw();
    if let Some(expected_denial) = expected_denial {
        let worth_proof::TransitionOutcome::Denied(denial) = opened else {
            panic!("post-seal selected-media substitution must deny at Serving open")
        };
        assert!(
            expected_denial(&denial.reason()),
            "unexpected Serving denial: {:?}",
            denial.reason()
        );
        return None;
    }
    let serving = match opened {
        worth_proof::TransitionOutcome::Success(serving) => serving,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("verified C8 custody Serving denial: {:?}", denial.reason())
        }
        worth_proof::TransitionOutcome::Stale(stale) => {
            panic!("verified C8 custody Serving stale: {:?}", stale.reason())
        }
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!("verified C8 custody Serving rebind: {:?}", rebind.reason())
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => match deferred {},
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!(
                "verified C8 custody Serving inspection/failure: {:?}",
                failure.cause()
            )
        }
    };
    if !require_checkpoint {
        return Some(serving);
    }
    let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x7b; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(checkpoint).into_raw()
    else {
        panic!("checkpoint on recovered Serving must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "checkpoint after recovered Serving: {outcome:?}"
    );
    serving.close();
    None
}

#[path = "declarations/serving_durability.rs"]
mod serving_durability;
pub use serving_durability::*;
