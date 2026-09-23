mod artifact_scope;
mod axes;
mod canonical;
mod held;
mod interleave;
mod page_variant;
mod qos;
mod ready;
mod reopen;
mod reopen_charge;
mod retention;
mod rewrite_scope;
mod scale;
mod scheduler;
mod span_layout;
mod sync;

pub(super) use canonical::canonical_child;
pub(super) use reopen::serving_child;

use std::collections::BTreeSet;
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy, ManifestEntryCapacity,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalOperationAllocationScope, PhysicalReadProtectionPolicy, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordId, PhysicalRecordInitialization,
    PhysicalRecordPlacementPolicy, PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind,
    PhysicalWalPolicy, RecordAppendBatch, RecordByteLimit, RecordReadLimits, SegmentPageCount,
    ServingPhysicalRuntime, WalSegmentByteLimit, WalSegmentInventoryLimit,
};

pub(super) fn limits() -> RecordReadLimits {
    RecordReadLimits::new(RecordByteLimit::new(64).unwrap())
}

pub(super) fn placement_for(
    format: AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(4).unwrap())
        .segment_pages(SegmentPageCount::new(8).unwrap())
        .admit(format)
        .unwrap()
}

pub(super) fn placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    placement_for(AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    ))
}

fn residency(
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
) -> AdmittedPhysicalRecordResidencyPolicy {
    residency_budget(format, resident, 16, 16, 8)
}

pub(super) fn rewrite_axis_residency(
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
) -> AdmittedPhysicalRecordResidencyPolicy {
    residency_budget(format, resident, 64, 32, 32)
}

fn residency_budget(
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
    frame_entries: u32,
    pinned_frames: u32,
    dirty_frames: u32,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;
    let checkpoint = 16 * 1024 * 1024;
    let scratch = checkpoint + 1024 * 1024;
    let metadata = 1024 * 1024;
    let dirty = 1024 * 1024;
    PhysicalRecordResidencyPolicy::builder()
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

pub(super) fn initialize(root: &Path) -> ServingPhysicalRuntime {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    assert_eq!(format.declaration().page_size().bytes(), 16 * 1024);
    initialize_with_protection(
        root,
        format,
        RESIDENT_BYTES,
        PhysicalReadProtectionPolicy::default(),
    )
}

pub(super) fn initialize_protection(
    root: &Path,
    protection: PhysicalReadProtectionPolicy,
) -> ServingPhysicalRuntime {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    initialize_with_protection(root, format, RESIDENT_BYTES, protection)
}

pub(super) fn initialize_with_format(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
) -> ServingPhysicalRuntime {
    initialize_with_protection(
        root,
        format,
        resident,
        PhysicalReadProtectionPolicy::default(),
    )
}

pub(super) fn initialize_with_protection(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
    protection: PhysicalReadProtectionPolicy,
) -> ServingPhysicalRuntime {
    initialize_with_wal(root, format, resident, protection, 512 * 1024)
}

pub(super) fn initialize_with_wal(
    root: &Path,
    format: AdmittedPhysicalRecordFormat,
    resident: u64,
    protection: PhysicalReadProtectionPolicy,
    wal_bytes: u64,
) -> ServingPhysicalRuntime {
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = super::media(root);
    let durability = super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(wal_bytes).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ),
    );
    let initialization =
        PhysicalRecordInitialization::new(format, placement_for(format), access, durability)
            .with_residency_policy(residency(format, resident))
            .with_read_protection_policy(protection);
    super::success(media.initialize_record_store(initialization))
}

pub(super) fn append(
    serving: &ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    bytes: &[u8],
) -> PhysicalRecordId {
    let mut material = [0x6A; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([bytes]).unwrap(),
            policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("append {ordinal} did not stay prepared"),
        TransitionOutcome::Denied(denial) => panic!("append {ordinal} denied: {denial:?}"),
        TransitionOutcome::Deferred(deferred) => panic!("append {ordinal} deferred: {deferred:?}"),
        TransitionOutcome::Stale(stale) => panic!("append {ordinal} stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => panic!("append {ordinal} rebind: {rebind:?}"),
        TransitionOutcome::Failed(failure) => panic!("append {ordinal} failed: {failure:?}"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(done) => {
            done.into_acknowledgment().record_ids().next().unwrap()
        }
        PhysicalMutationOutcome::ProvenNoEffect(fate) => panic!(
            "append {ordinal} had no effect: {:?} charged={}",
            fate.cause(),
            serving.certification_charged_growth_bytes()
        ),
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!(
                "append {ordinal} became indeterminate at {:?}",
                fate.stage()
            )
        }
    }
}

pub(super) fn append_copies(
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
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter(records).unwrap(),
            policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("append {ordinal} did not stay prepared"),
        TransitionOutcome::Denied(denial) => panic!("append {ordinal} denied: {denial:?}"),
        TransitionOutcome::Deferred(deferred) => panic!("append {ordinal} deferred: {deferred:?}"),
        TransitionOutcome::Stale(stale) => panic!("append {ordinal} stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => panic!("append {ordinal} rebind: {rebind:?}"),
        TransitionOutcome::Failed(failure) => panic!("append {ordinal} failed: {failure:?}"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(done) => {
            Some(done.into_acknowledgment().record_ids().next().unwrap())
        }
        PhysicalMutationOutcome::ProvenNoEffect(_) => None,
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("batch {ordinal} became indeterminate at {:?}", fate.stage())
        }
    }
}

pub(super) fn segment_ids(root: &Path) -> BTreeSet<u64> {
    let mut ids = BTreeSet::new();
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

pub(super) fn segment_files(root: &Path) -> BTreeSet<String> {
    std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".pages"))
        .collect()
}

pub(super) fn checkpoint(serving: &ServingPhysicalRuntime, ordinal: u64) {
    let mut key = [0x6C; 32];
    key[..8].copy_from_slice(&ordinal.to_le_bytes());
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Denied(_) => panic!("checkpoint {ordinal} denied"),
        TransitionOutcome::Deferred(_) => panic!("checkpoint {ordinal} deferred"),
        TransitionOutcome::Stale(_) => panic!("checkpoint {ordinal} stale"),
        TransitionOutcome::RebindRequired(_) => panic!("checkpoint {ordinal} rebind"),
        TransitionOutcome::Failed(_) => panic!("checkpoint {ordinal} failed"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => {}
        PhysicalCheckpointOutcome::ProvenNoEffect(effect) => {
            panic!("checkpoint {ordinal} had no effect: {:?}", effect.cause())
        }
        PhysicalCheckpointOutcome::Indeterminate(_) => {
            panic!("checkpoint {ordinal} became indeterminate")
        }
    }
}

const RESIDENT_BYTES: u64 = 64 * 1024;

fn nonzero(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}

fn nonzero_count(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}
