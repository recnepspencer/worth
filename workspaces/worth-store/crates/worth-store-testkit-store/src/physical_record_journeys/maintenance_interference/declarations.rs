pub use crate::physical_record_journeys::maintenance_interference::canonical::canonical_child;
pub use crate::physical_record_journeys::maintenance_interference::reopen::serving_child;

pub use std::collections::BTreeSet;
pub use std::path::Path;
pub use {std::num::NonZeroU32, std::num::NonZeroU64};

pub use worth_proof::TransitionOutcome;
pub use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalReadProtectionPolicy,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind,
    worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::SegmentPageCount,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};

pub fn limits() -> RecordReadLimits {
    worth_store::physical_runtime::RecordReadLimits::new(
        worth_store::physical_runtime::RecordByteLimit::new(64).unwrap(),
    )
}

pub fn placement_for(
    format: AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(4).unwrap())
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(8).unwrap())
        .admit(format)
        .unwrap()
}

pub fn placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    placement_for(
        worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
            worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
                .admit()
                .unwrap(),
        ),
    )
}

pub fn residency(
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
) -> AdmittedPhysicalRecordResidencyPolicy {
    residency_budget(format, resident, 16, 16, 8)
}

pub fn rewrite_axis_residency(
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
) -> AdmittedPhysicalRecordResidencyPolicy {
    residency_budget(format, resident, 64, 32, 32)
}

pub fn residency_budget(
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
    frame_entries: u32,
    pinned_frames: u32,
    dirty_frames: u32,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope;
    use worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation;
    let checkpoint = 16 * 1024 * 1024;
    let scratch = checkpoint + 1024 * 1024;
    let metadata = 1024 * 1024;
    let dirty = 1024 * 1024;
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero(scratch + metadata + dirty + resident))
        .resident_bytes(nonzero(resident))
        .metadata_bytes(nonzero(metadata))
        .frame_entries(nonzero_count(frame_entries))
        .pinned_frames(nonzero_count(pinned_frames))
        .pin_leases(nonzero_count(64))
        .dirty_frames(nonzero_count(dirty_frames))
        .dirty_replacement_bytes(nonzero(dirty))
        .operation_bytes(nonzero(scratch))
        .scope_bytes(Scope::ForegroundRead, nonzero(checkpoint))
        .scope_bytes(Scope::ForegroundWrite, nonzero(checkpoint))
        .scope_bytes(Scope::Recovery, nonzero(checkpoint))
        .scope_bytes(Scope::Scrub, nonzero(checkpoint))
        .scope_bytes(Scope::Maintenance, nonzero(checkpoint))
        .scope_bytes(Scope::Verification, nonzero(checkpoint))
        .scope_bytes(Scope::Blob, nonzero(checkpoint))
        .speculative_frames(Speculation::Prefetch, nonzero_count(8))
        .speculative_frames(Speculation::ReadAhead, nonzero_count(8))
        .speculative_frames(Speculation::WriteBehind, nonzero_count(4))
        .admit(format)
        .into_result()
        .expect("the resident budget must admit the page size")
}

pub fn initialize(root: &Path) -> ServingPhysicalRuntime {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    assert_eq!(format.declaration().page_size().bytes(), 16 * 1024);
    initialize_with_protection(
        root,
        format,
        RESIDENT_BYTES,
        worth_store::physical_runtime::PhysicalReadProtectionPolicy::default(),
    )
}

pub fn initialize_protection(
    root: &Path,
    protection: PhysicalReadProtectionPolicy,
) -> ServingPhysicalRuntime {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    initialize_with_protection(root, format, RESIDENT_BYTES, protection)
}

pub fn initialize_with_format(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
) -> ServingPhysicalRuntime {
    initialize_with_protection(
        root,
        format,
        resident,
        worth_store::physical_runtime::PhysicalReadProtectionPolicy::default(),
    )
}

pub fn initialize_with_protection(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
    protection: PhysicalReadProtectionPolicy,
) -> ServingPhysicalRuntime {
    initialize_with_wal(root, format, resident, protection, 512 * 1024)
}

pub fn initialize_with_wal(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
    protection: PhysicalReadProtectionPolicy,
    wal_bytes: u64,
) -> ServingPhysicalRuntime {
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let media = crate::physical_record_journeys::media(root);
    let durability = crate::physical_record_journeys::durability_with_wal_policy(
        &media,
        worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(
                std::num::NonZeroU64::new(wal_bytes).unwrap(),
            ),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                std::num::NonZeroU32::new(64).unwrap(),
            ),
        ),
    );
    let initialization = worth_store::physical_runtime::PhysicalRecordInitialization::new(
        format,
        placement_for(format),
        access,
        durability,
    )
    .with_residency_policy(residency(format, resident))
    .with_read_protection_policy(protection);
    crate::physical_record_journeys::success(media.initialize_record_store(initialization))
}

pub fn append(
    serving: &ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    bytes: &[u8],
) -> PhysicalRecordId {
    let mut material = [0x6A; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([bytes]).unwrap(),
            policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("append {ordinal} did not stay prepared")
        }
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("append {ordinal} denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            panic!("append {ordinal} deferred: {deferred:?}")
        }
        worth_proof::TransitionOutcome::Stale(stale) => panic!("append {ordinal} stale: {stale:?}"),
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!("append {ordinal} rebind: {rebind:?}")
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("append {ordinal} failed: {failure:?}")
        }
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(done) => {
            done.into_acknowledgment().record_ids().next().unwrap()
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => panic!(
            "append {ordinal} had no effect: {:?} charged={}",
            fate.cause(),
            serving.certification_charged_growth_bytes()
        ),
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!(
                "append {ordinal} became indeterminate at {:?}",
                fate.stage()
            )
        }
    }
}

pub fn append_copies(
    serving: &ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    bytes: &[u8],
    copies: usize,
) -> Option<PhysicalRecordId> {
    let records = vec![bytes; copies];
    let mut material = [0x6A; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter(records).unwrap(),
            policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("append {ordinal} did not stay prepared")
        }
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("append {ordinal} denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            panic!("append {ordinal} deferred: {deferred:?}")
        }
        worth_proof::TransitionOutcome::Stale(stale) => panic!("append {ordinal} stale: {stale:?}"),
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!("append {ordinal} rebind: {rebind:?}")
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("append {ordinal} failed: {failure:?}")
        }
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(done) => {
            Some(done.into_acknowledgment().record_ids().next().unwrap())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(_) => None,
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("batch {ordinal} became indeterminate at {:?}", fate.stage())
        }
    }
}

pub fn segment_ids(root: &Path) -> BTreeSet<u64> {
    let mut ids = std::collections::BTreeSet::new();
    let entries = std::fs::read_dir(root.join("families/records/segments")).unwrap();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix("segment-") else {
            continue;
        };
        let id = u64::from_str_radix(&rest[..16], 16).unwrap();
        ids.insert(id);
    }
    ids
}

pub fn segment_files(root: &Path) -> BTreeSet<String> {
    std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".pages"))
        .collect()
}

pub fn checkpoint(serving: &ServingPhysicalRuntime, ordinal: u64) {
    let mut key = [0x6C; 32];
    key[..8].copy_from_slice(&ordinal.to_le_bytes());
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        worth_proof::TransitionOutcome::Denied(_) => panic!("checkpoint {ordinal} denied"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("checkpoint {ordinal} deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("checkpoint {ordinal} stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => panic!("checkpoint {ordinal} rebind"),
        worth_proof::TransitionOutcome::Failed(_) => panic!("checkpoint {ordinal} failed"),
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_) => {}
        worth_store::physical_runtime::PhysicalCheckpointOutcome::ProvenNoEffect(effect) => {
            panic!("checkpoint {ordinal} had no effect: {:?}", effect.cause())
        }
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Indeterminate(_) => {
            panic!("checkpoint {ordinal} became indeterminate")
        }
    }
}

pub const RESIDENT_BYTES: u64 = 64 * 1024;

pub fn nonzero(value: u64) -> NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

pub fn nonzero_count(value: u32) -> NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}
