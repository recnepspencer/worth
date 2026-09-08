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
    PhysicalMutationHandle, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordInitialization, PhysicalRecordPlacementPolicy,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, RecordAppendBatch,
    RecordServingTerminalPosture, RetainedWalTailLimit, SegmentPageCount, ServingPhysicalRuntime,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::FilesystemAccessPosture;

use super::production_profile::ProductionWorldProfile;
use super::ClosedStoreProcessManifest;

pub(crate) fn produce_closed_store(
    root: &Path,
    profile: ProductionWorldProfile,
) -> Result<ClosedStoreProcessManifest, String> {
    let (serving, placement) = initialize_store(root, profile)?;
    populate_and_close(root, profile, serving, placement)
}

pub(super) fn initialize_store(
    root: &Path,
    profile: ProductionWorldProfile,
) -> Result<
    (
        ServingPhysicalRuntime,
        worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ),
    String,
> {
    std::fs::create_dir(root).map_err(|error| format!("create Store root: {error}"))?;
    let runtime = PhysicalStore::admit(
        PhysicalRuntimeAdmission::new(root).map_err(|error| format!("admit root: {error:?}"))?,
    )
    .map_err(|error| format!("admit runtime: {error:?}"))?;
    let media = admit_media(runtime)?;
    let durability = admit_durability(&media)?;
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(profile.page_size())
            .admit()
            .map_err(|error| format!("admit format: {error:?}"))?,
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(16).expect("nonzero capacity"))
        .segment_pages(SegmentPageCount::new(4).expect("nonzero segment pages"))
        .admit(format)
        .map_err(|error| format!("admit placement: {error:?}"))?;
    let access = PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .map_err(|error| format!("admit access: {error:?}"))?;
    let serving = admit_records(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_residency_policy(profile.residency(format)),
        ),
    )?;
    Ok((serving, placement))
}

fn populate_and_close(
    root: &Path,
    profile: ProductionWorldProfile,
    serving: ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
) -> Result<ClosedStoreProcessManifest, String> {
    let mut records = Vec::new();
    for batch in 0..profile.batches() {
        if matches!(
            profile,
            ProductionWorldProfile::Primary16KiB | ProductionWorldProfile::ReusedTails16KiB
        ) && batch + 1 == profile.batches() / 2
        {
            let gate = serving.pause_physical_mutation_at(
                worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWritebackAdmissionBeforeEffect,
            );
            let mutation = start_batch(&serving, placement, profile, batch)?;
            if !gate.await_arrival() {
                gate.release();
                return Err(
                    "production mutation did not reach dirty writeback admission".to_owned(),
                );
            }
            let checkpoint = publish_checkpoint(&serving, true);
            gate.release();
            checkpoint?;
            records.extend(super::production_record::ProducedRecord::completed(
                &require_mutation(mutation.wait(), batch)?,
                batch,
                profile,
            ));
        } else {
            let completed = require_mutation(
                start_batch(&serving, placement, profile, batch)?.wait(),
                batch,
            )?;
            records.extend(super::production_record::ProducedRecord::completed(
                &completed, batch, profile,
            ));
            if matches!(
                profile,
                ProductionWorldProfile::Pages32KiB | ProductionWorldProfile::Pages64KiB
            ) && batch == 0
            {
                publish_checkpoint(&serving, false)?;
            }
        }
    }
    let resident = serving.residency_observation();
    assert!(resident.counters().peak_resident_bytes() <= profile.resident_bytes());
    assert!(resident.counters().evictions() > 0);
    let shutdown = serving.close();
    if shutdown.records().posture() == RecordServingTerminalPosture::InspectionRequired
        || shutdown.checkpoint().requires_inspection()
        || shutdown.residency().requires_inspection()
        || shutdown.work().drain().requires_inspection()
        || shutdown.durability_closeout().requires_inspection()
    {
        return Err("clean Store close required inspection".to_owned());
    }
    let mut manifest = ClosedStoreProcessManifest::observe(root)
        .map_err(|error| format!("observe closed Store manifest: {error:?}"))?;
    manifest.records = records;
    super::production_world_shape::require_shape(root, &manifest, profile);
    if matches!(
        profile,
        ProductionWorldProfile::Primary16KiB | ProductionWorldProfile::ReusedTails16KiB
    ) {
        assert!(
            manifest.byte_count() >= 32 * profile.resident_bytes(),
            "production scale: {} occupied bytes / {} resident bytes",
            manifest.byte_count(),
            profile.resident_bytes()
        );
    }
    println!(
        "C9 producer profile={} occupied_bytes={} resident_limit={} peak_resident={} evictions={}",
        profile.label(),
        manifest.byte_count(),
        profile.resident_bytes(),
        resident.counters().peak_resident_bytes(),
        resident.counters().evictions()
    );
    Ok(manifest)
}

fn publish_checkpoint(serving: &ServingPhysicalRuntime, require_dirty: bool) -> Result<(), String> {
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
        PhysicalCheckpointOutcome::Completed(completed)
            if !require_dirty || completed.dirty_records() > 0 =>
        {
            Ok(())
        }
        outcome => Err(format!(
            "production dirty checkpoint did not complete: {outcome:?}"
        )),
    }
}

pub(super) fn start_batch(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    profile: ProductionWorldProfile,
    ordinal: usize,
) -> Result<PhysicalMutationHandle, String> {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
            [ordinal as u8 + 1; 32],
        ))
        .map_err(|error| format!("issue idempotency key: {error:?}"))?;
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000)
            .ok_or_else(|| "admit deadline".to_owned())?,
    );
    let mut payloads = (0..profile.inline_records_per_batch(ordinal))
        .map(|record| vec![(ordinal * 17 + record) as u8; profile.inline_record_bytes()])
        .collect::<Vec<_>>();
    if matches!(
        profile,
        ProductionWorldProfile::Primary16KiB | ProductionWorldProfile::ReusedTails16KiB
    ) {
        payloads.push(vec![0xC9 ^ ordinal as u8; 64 * 1024]);
    }
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter(payloads.iter().map(Vec::as_slice))
                .map_err(|error| format!("admit record batch: {error:?}"))?,
            placement,
            request,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Denied(denial) => {
            return Err(format!(
                "prepare durable append {ordinal} denied: {denial:?}"
            ))
        }
        _ => return Err(format!("prepare durable append {ordinal} did not succeed")),
    };
    Ok(prepared.start())
}

fn require_mutation(
    outcome: PhysicalMutationOutcome,
    ordinal: usize,
) -> Result<worth_store::physical_runtime::CompletedPhysicalMutation, String> {
    match outcome {
        PhysicalMutationOutcome::Completed(completed) => Ok(completed),
        PhysicalMutationOutcome::ProvenNoEffect(fate) => Err(format!(
            "execute durable mutation {ordinal} no effect: {fate:?}"
        )),
        PhysicalMutationOutcome::Indeterminate(fate) => Err(format!(
            "execute durable mutation {ordinal} indeterminate: {fate:?}"
        )),
    }
}

pub(super) fn admit_media(
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

pub(super) fn admit_durability(
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
            WalSegmentByteLimit::new(NonZeroU64::new(2 * 1024 * 1024).expect("nonzero WAL")),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1_024).expect("nonzero inventory")),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).expect("nonzero retention")),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1_024).expect("nonzero pending")),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4_096).expect("nonzero bindings")),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(512 * 1024).expect("nonzero memory")),
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
