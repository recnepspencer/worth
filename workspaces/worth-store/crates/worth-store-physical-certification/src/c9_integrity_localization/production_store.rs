use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalDurabilityPolicy, CheckpointMemoryLimit, FilesystemMediaAdmission,
    GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, ManifestEntryCapacity, MediaOwnedPhysicalRuntime,
    PendingUnresolvedMutationLimit, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointPolicy, PhysicalCheckpointRequest,
    PhysicalDurabilityDeclaration, PhysicalIdempotencyPolicy, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordInitialization, PhysicalRecordPlacementPolicy,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, RecordAppendBatch,
    RecordServingTerminalPosture, RetainedWalTailLimit, ServingPhysicalRuntime,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::FilesystemAccessPosture;

use super::ClosedStoreProcessManifest;

pub(crate) fn produce_closed_store(root: &Path) -> Result<ClosedStoreProcessManifest, String> {
    produce_records(root, &[b"c9-production-root"])
}

pub(super) fn produce_closed_store_with_extent(
    root: &Path,
) -> Result<ClosedStoreProcessManifest, String> {
    let extent = vec![0x5a; 40_000];
    produce_records(root, &[b"ordinary-inline-record", &extent])
}

fn produce_records(root: &Path, records: &[&[u8]]) -> Result<ClosedStoreProcessManifest, String> {
    std::fs::create_dir(root).map_err(|error| format!("create Store root: {error}"))?;
    let runtime = PhysicalStore::admit(
        PhysicalRuntimeAdmission::new(root).map_err(|error| format!("admit root: {error:?}"))?,
    )
    .map_err(|error| format!("admit runtime: {error:?}"))?;
    let media = admit_media(runtime)?;
    let durability = admit_durability(&media)?;
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .admit()
            .map_err(|error| format!("admit format: {error:?}"))?,
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).expect("nonzero capacity"))
        .admit(format)
        .map_err(|error| format!("admit placement: {error:?}"))?;
    let access = PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .map_err(|error| format!("admit access: {error:?}"))?;
    let serving = admit_records(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )?;
    publish_one_root(&serving, placement, records)?;
    publish_checkpoint(&serving)?;
    let shutdown = serving.close();
    if shutdown.records().posture() == RecordServingTerminalPosture::InspectionRequired
        || shutdown.checkpoint().requires_inspection()
        || shutdown.residency().requires_inspection()
        || shutdown.work().drain().requires_inspection()
        || shutdown.durability_closeout().requires_inspection()
    {
        return Err("clean Store close required inspection".to_owned());
    }
    ClosedStoreProcessManifest::observe(root)
        .map_err(|error| format!("observe closed Store manifest: {error:?}"))
}

fn publish_checkpoint(serving: &ServingPhysicalRuntime) -> Result<(), String> {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x39; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .ok_or_else(|| "admit checkpoint deadline".to_owned())?,
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => return Err("start production checkpoint did not succeed".to_owned()),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => Ok(()),
        _ => Err("production checkpoint did not complete".to_owned()),
    }
}

fn publish_one_root(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    records: &[&[u8]],
) -> Result<(), String> {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xC9; 32]))
        .map_err(|error| format!("issue idempotency key: {error:?}"))?;
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(5_000)
            .ok_or_else(|| "admit deadline".to_owned())?,
    );
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter(records.iter().copied())
                .map_err(|error| format!("admit record batch: {error:?}"))?,
            placement,
            request,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => return Err("prepare durable append did not succeed".to_owned()),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(_) => Ok(()),
        _ => Err("execute durable mutation did not complete".to_owned()),
    }
}

fn admit_media(
    runtime: worth_store::physical_runtime::AdmittedPhysicalRuntime,
) -> Result<MediaOwnedPhysicalRuntime, String> {
    match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => Ok(media),
        _ => Err("admit filesystem media did not succeed".to_owned()),
    }
}

fn admit_durability(
    media: &MediaOwnedPhysicalRuntime,
) -> Result<AdmittedPhysicalDurabilityPolicy, String> {
    let basis = media
        .physical_durability_admission_basis()
        .map_err(|error| format!("admit durability basis: {error:?}"))?;
    let declaration = PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).expect("nonzero group limit")),
            GroupCommitDelay::new(NonZeroU64::new(1).expect("nonzero group delay")),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(16 * 1024 * 1024).expect("nonzero WAL")),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1_024).expect("nonzero inventory")),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).expect("nonzero retention")),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1_024).expect("nonzero pending")),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4_096).expect("nonzero bindings")),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 * 1024 * 1024).expect("nonzero memory")),
            RetainedWalTailLimit::new(NonZeroU64::new(64 * 1024 * 1024).expect("nonzero tail")),
        ));
    match declaration.admit(basis).into_raw() {
        TransitionOutcome::Success(policy) => Ok(policy),
        _ => Err("admit durability policy did not succeed".to_owned()),
    }
}

fn admit_records(
    outcome: worth_store::physical_runtime::RecordStoreInitializationOutcome,
) -> Result<ServingPhysicalRuntime, String> {
    match outcome.into_raw() {
        TransitionOutcome::Success(serving) => Ok(serving),
        _ => Err("initialize record Store did not succeed".to_owned()),
    }
}
