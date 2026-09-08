use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

macro_rules! initialize_record_store {
    ($media:expr, |$policy:ident| $request:expr $(,)?) => {{
        let media_owner = $media;
        let $policy = $crate::durability(&media_owner);
        media_owner.initialize_record_store($request)
    }};
}

macro_rules! open_record_store {
    ($media:expr, |$policy:ident| $request:expr $(,)?) => {{
        let media_owner = $media;
        let $policy = $crate::durability(&media_owner);
        media_owner.open_record_store($request)
    }};
}

#[path = "physical_record_journeys/allocation_probe.rs"]
mod allocation_probe;
#[path = "physical_record_journeys/baseline_admission.rs"]
mod baseline_admission;
#[path = "physical_record_journeys/batch_admission.rs"]
mod batch_admission;
#[path = "physical_record_journeys/bootstrap_faults.rs"]
mod bootstrap_faults;
#[path = "physical_record_journeys/child_process.rs"]
mod child_process;
#[path = "physical_record_journeys/close_phase_crash.rs"]
mod close_phase_crash;
#[path = "physical_record_journeys/configuration_boundaries.rs"]
mod configuration_boundaries;
#[path = "physical_record_journeys/courtroom_child.rs"]
mod courtroom_child;
#[path = "c5/courtroom_evidence_support.rs"]
mod courtroom_evidence_support;
#[path = "c5/courtroom_oracle.rs"]
mod courtroom_oracle;
#[path = "physical_record_journeys/durability_admission.rs"]
mod durability_admission;
#[path = "physical_record_journeys/durable_frame_oracle.rs"]
mod durable_frame_oracle;
#[path = "physical_record_journeys/durable_publication/mod.rs"]
mod durable_publication;
#[path = "c5/extent_child.rs"]
mod extent_child;
#[path = "physical_record_journeys/extent_streaming.rs"]
mod extent_streaming;
#[path = "physical_record_journeys/format_readmission.rs"]
mod format_readmission;
#[path = "physical_record_journeys/foundational_evidence.rs"]
mod foundational_evidence;
#[path = "physical_record_journeys/generation_policy_truth.rs"]
mod generation_policy_truth;
#[path = "physical_record_journeys/identity_process.rs"]
mod identity_process;
#[path = "physical_record_journeys/integrity_scrub.rs"]
mod integrity_scrub;
#[path = "physical_record_journeys/locator_free_space.rs"]
mod locator_free_space;
#[path = "physical_record_journeys/manifest_fixture.rs"]
mod manifest_fixture;
#[path = "physical_record_journeys/manifest_scale.rs"]
mod manifest_scale;
#[path = "c5/observer.rs"]
mod observer;
#[path = "physical_record_journeys/page_packing_oracle.rs"]
mod page_packing_oracle;
#[path = "physical_record_journeys/physical_work/mod.rs"]
mod physical_work;
#[path = "c5/courtrooms.rs"]
mod production_courtrooms;
#[path = "physical_record_journeys/publication_reopener.rs"]
mod publication_reopener;
#[path = "physical_record_journeys/read_boundaries.rs"]
mod read_boundaries;
#[path = "physical_record_journeys/record_chunk_views.rs"]
mod record_chunk_views;
#[path = "physical_record_journeys/residency_pressure_processes.rs"]
mod residency_pressure_processes;
#[path = "physical_record_journeys/residency_writeback_fresh_reopen.rs"]
mod residency_writeback_fresh_reopen;
#[path = "c5/scale_invalid_worlds.rs"]
mod scale_invalid_worlds;
#[path = "c5/scale_policy_evolution.rs"]
mod scale_policy_evolution;
#[path = "c5/scale.rs"]
mod scale_support;
#[path = "physical_record_journeys/scan_journeys.rs"]
mod scan_journeys;
#[path = "physical_record_journeys/scenario_configuration.rs"]
mod scenario_configuration;
#[path = "physical_record_journeys/scenario_evidence.rs"]
mod scenario_evidence;
#[path = "physical_record_journeys/scenario_process_evidence.rs"]
mod scenario_process_evidence;
#[cfg(feature = "recovery-runtime-owner")]
#[path = "physical_record_journeys/scheduled_reopen_admission.rs"]
mod scheduled_reopen_admission;
#[path = "physical_record_journeys/segment_journeys.rs"]
mod segment_journeys;
#[path = "physical_record_journeys/segment_truth.rs"]
mod segment_truth;
#[path = "physical_record_journeys/serving_lifecycle.rs"]
mod serving_lifecycle;
#[path = "physical_record_journeys/stream_fixture.rs"]
mod stream_fixture;
#[path = "physical_record_journeys/successor_scope_admission.rs"]
mod successor_scope_admission;

use child_process::run_child;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalDurabilityPolicy, AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy,
    AdmittedRecordPlacementPolicy, CheckpointMemoryLimit, FilesystemMediaAdmission,
    GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, ManifestEntryCapacity, MediaOwnedPhysicalRuntime,
    PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration,
    PhysicalIdempotencyPolicy, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalRecordPlacementPolicy,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, RecordBootstrapDenial,
    RecordReadObservation, RecordReadSession, RecordServingAdmissionOutcome,
    RecordStoreInitializationDenial, RecordStoreOpenDenial, RetainedWalTailLimit,
    ServingPhysicalRuntime, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::FilesystemAccessPosture;

fn configuration() -> (
    AdmittedPhysicalRecordFormat,
    AdmittedRecordPlacementPolicy,
    AdmittedRecordAccessPolicy,
) {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    (format, placement, access)
}

fn media(root: &Path) -> MediaOwnedPhysicalRuntime {
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("real filesystem media admission must succeed"),
    }
}

fn durability(media: &MediaOwnedPhysicalRuntime) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_group_limit(media, NonZeroU32::new(32).unwrap())
}

fn durability_with_group_limit(
    media: &MediaOwnedPhysicalRuntime,
    group_limit: NonZeroU32,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_pending_limit(media, group_limit, NonZeroU32::new(1_024).unwrap())
}

fn durability_with_pending_limit(
    media: &MediaOwnedPhysicalRuntime,
    group_limit: NonZeroU32,
    pending_limit: NonZeroU32,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_limits(
        media,
        group_limit,
        pending_limit,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1_024).unwrap()),
        ),
    )
}

fn durability_with_wal_policy(
    media: &MediaOwnedPhysicalRuntime,
    wal: PhysicalWalPolicy,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_limits(
        media,
        NonZeroU32::new(32).unwrap(),
        NonZeroU32::new(1_024).unwrap(),
        wal,
    )
}

fn durability_with_limits(
    media: &MediaOwnedPhysicalRuntime,
    group_limit: NonZeroU32,
    pending_limit: NonZeroU32,
    wal: PhysicalWalPolicy,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_idempotency_and_wal_limits(
        media,
        group_limit,
        pending_limit,
        NonZeroU32::new(4_096).unwrap(),
        wal,
    )
}

fn durability_with_idempotency_limits(
    media: &MediaOwnedPhysicalRuntime,
    pending_limit: NonZeroU32,
    live_limit: NonZeroU32,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_idempotency_and_wal_limits(
        media,
        NonZeroU32::new(32).unwrap(),
        pending_limit,
        live_limit,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1_024).unwrap()),
        ),
    )
}

fn durability_with_idempotency_and_wal_limits(
    media: &MediaOwnedPhysicalRuntime,
    group_limit: NonZeroU32,
    pending_limit: NonZeroU32,
    live_limit: NonZeroU32,
    wal: PhysicalWalPolicy,
) -> AdmittedPhysicalDurabilityPolicy {
    let basis = media.physical_durability_admission_basis().unwrap();
    match PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(group_limit),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(wal)
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(pending_limit),
            LiveIdempotencyBindingLimit::new(live_limit),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 * 1024 * 1024).unwrap()),
        ))
        .admit(basis)
        .into_raw()
    {
        TransitionOutcome::Success(policy) => policy,
        _ => panic!("qualified test media must admit the explicit durability policy"),
    }
}

fn serving_from_initialization(root: &Path) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )
}

fn serving_from_open(root: &Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    success(media.open_record_store(PhysicalRecordOpen::new(format, access, durability)))
}

trait BootstrapDenialReason {
    fn reason(&self) -> RecordBootstrapDenial;
}

impl BootstrapDenialReason for RecordStoreInitializationDenial {
    fn reason(&self) -> RecordBootstrapDenial {
        self.reason()
    }
}

impl BootstrapDenialReason for RecordStoreOpenDenial {
    fn reason(&self) -> RecordBootstrapDenial {
        self.reason()
    }
}

fn success<Denial>(outcome: RecordServingAdmissionOutcome<Denial>) -> ServingPhysicalRuntime
where
    Denial: BootstrapDenialReason,
{
    match outcome.into_raw() {
        TransitionOutcome::Success(serving) => serving,
        TransitionOutcome::Denied(denial) => {
            panic!(
                "record-serving progression must succeed; denied by {:?}",
                denial.reason()
            )
        }
        TransitionOutcome::Deferred(deferred) => match deferred {},
        TransitionOutcome::Stale(stale) => {
            panic!(
                "record-serving progression must succeed; stale because {:?}",
                stale.reason()
            )
        }
        TransitionOutcome::RebindRequired(rebind) => {
            panic!(
                "record-serving progression must succeed; rebind required because {:?}",
                rebind.reason()
            )
        }
        TransitionOutcome::Failed(failure) => {
            panic!(
                "record-serving progression must succeed; inspection required because {:?}",
                failure.cause()
            )
        }
    }
}

fn read_record(
    mut session: RecordReadSession,
    expected_bytes: usize,
) -> (Vec<u8>, RecordReadObservation) {
    let mut bytes = vec![0_u8; expected_bytes];
    let mut completed = 0;
    while completed < bytes.len() {
        let count = session.read_next(&mut bytes[completed..]).unwrap();
        assert!(count > 0, "the record ended before its declared length");
        completed += count;
    }
    assert_eq!(session.read_next(&mut [0_u8; 1]).unwrap(), 0);
    (bytes, session.observation())
}
