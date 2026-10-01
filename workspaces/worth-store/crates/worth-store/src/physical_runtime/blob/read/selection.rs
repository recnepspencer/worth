use std::num::NonZeroU64;

use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobRecordDenial, BlobSessionDeclarationV1,
};

use crate::physical_runtime::{
    layout::{PhysicalIndexPointKey, PhysicalLayoutAccess},
    BlobPhysicalAllocation, PhysicalRecordId, PhysicalRecordReader, RecordByteLimit,
    RecordCountLimit, RecordReadLimits, RecordScanOutcome, RecordScanRequest,
    ServingPhysicalRuntime,
};
use worth_store_contracts::DurableArtifactFamilyId;

use super::super::{BlobGeneration, BlobObjectId, BlobSessionId};
use super::{
    traversal, AdmittedBlobScope, BlobReadFailure, BlobReadLimits, BlobReadObservation,
    BlobReadOpenFailure, BlobReadSession, PublishedBlobGeneration,
};

const SCAN_SCRATCH_BYTES: usize = 4096;
const READ_CHARGE_BYTES: u64 = SCAN_SCRATCH_BYTES as u64 + 7 * 512 * 1024 + 1024 * 1024;
const BLOB_MAGIC: &[u8; 8] = b"WRC11BLB";
const MAX_SELECTED_CONTROL_BYTES: u64 = 48 + 188;

pub(super) fn open<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    published: PublishedBlobGeneration,
    scope: &AdmittedBlobScope,
    offset: u64,
    length: u64,
    limits: BlobReadLimits,
) -> Result<BlobReadSession<'runtime>, BlobReadOpenFailure> {
    if published.store() != runtime.store_identity() {
        return Err(BlobReadOpenFailure::ForeignStore);
    }
    let (reader, allocation, publication) = selected_publication(
        runtime,
        published.object().bytes(),
        published.generation().sequence(),
        Some(published.session().bytes()),
        scope,
        limits,
    )?;
    let end = offset
        .checked_add(length)
        .filter(|end| *end <= publication.total_bytes())
        .ok_or(BlobReadOpenFailure::RangeOutOfBounds)?;
    let mut session = BlobReadSession {
        reader,
        publication,
        cursor: offset,
        end,
        nodes: Vec::new(),
        chunk: None,
        damaged_chunk_edge: None,
        observation: BlobReadObservation::default(),
        _allocation: allocation,
    };
    traversal::validate_root(&mut session).map_err(BlobReadOpenFailure::Read)?;
    Ok(session)
}

pub(super) fn resolve(
    runtime: &ServingPhysicalRuntime,
    object: [u8; 16],
    generation: u64,
    scope: &AdmittedBlobScope,
    limits: BlobReadLimits,
) -> Result<PublishedBlobGeneration, BlobReadOpenFailure> {
    let (_, _, selected) = selected_publication(runtime, object, generation, None, scope, limits)?;
    let store = runtime.store_identity();
    Ok(PublishedBlobGeneration::from_completed_publication(
        store,
        BlobSessionId::from_selected(selected.session()),
        BlobObjectId::from_selected(store, selected.object()),
        BlobGeneration::published(selected.generation()),
    ))
}

fn selected_publication<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    object: [u8; 16],
    generation: u64,
    expected_session: Option<[u8; 16]>,
    scope: &AdmittedBlobScope,
    _limits: BlobReadLimits,
) -> Result<
    (
        PhysicalRecordReader,
        BlobPhysicalAllocation<'runtime>,
        BlobGenerationPublicationV1,
    ),
    BlobReadOpenFailure,
> {
    if object == [0; 16] || generation == 0 {
        return Err(BlobReadOpenFailure::InvalidRequestedIdentity);
    }
    let allocation = runtime
        .physical_allocations()
        .admit_blob(NonZeroU64::new(READ_CHARGE_BYTES).expect("positive read charge"))
        .map_err(BlobReadOpenFailure::Allocation)?;
    let reader = runtime
        .records()
        .map_err(BlobReadOpenFailure::RootProtection)?;
    let layouts =
        PhysicalLayoutAccess::from_reader(runtime, reader).map_err(BlobReadOpenFailure::Layout)?;
    let key =
        PhysicalIndexPointKey::selected_blob_catalog(runtime.store_identity(), object, generation)
            .map_err(BlobReadOpenFailure::InvalidPointKey)?;
    let selected = layouts
        .btree(DurableArtifactFamilyId::BlobCatalog)
        .and_then(|tree| tree.point(key))
        .map_err(BlobReadOpenFailure::Layout)?
        .selected_record()
        .ok_or(BlobReadOpenFailure::PublicationNotFound)?;
    let reader = layouts.into_reader();
    let publication = read_selected_publication(&reader, selected)?;
    if publication.object() != object
        || publication.generation() != generation
        || publication.store() != reader.store_identity().bytes()
        || expected_session.is_some_and(|session| publication.session() != session)
    {
        return Err(BlobReadOpenFailure::ConflictingPublication);
    }
    if publication.key_scope() != scope.fingerprint() {
        return Err(BlobReadOpenFailure::ScopeMismatch);
    }
    Ok((reader, allocation, publication))
}

fn read_selected_publication(
    reader: &PhysicalRecordReader,
    selected: worth_store_physical_format::PersistedRecordIdentity,
) -> Result<BlobGenerationPublicationV1, BlobReadOpenFailure> {
    let limit = RecordByteLimit::new(MAX_SELECTED_CONTROL_BYTES as u32)
        .expect("publication frame bound is nonzero");
    let mut opened = reader
        .open(
            PhysicalRecordId::from_persisted(selected),
            RecordReadLimits::new(limit),
        )
        .map_err(|cause| BlobReadOpenFailure::Read(BlobReadFailure::RecordRead(cause)))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(MAX_SELECTED_CONTROL_BYTES as usize)
        .map_err(|_| BlobReadOpenFailure::ScratchUnavailable)?;
    let mut scratch = [0_u8; MAX_SELECTED_CONTROL_BYTES as usize];
    loop {
        let count = opened
            .read_next(&mut scratch)
            .map_err(|cause| BlobReadOpenFailure::Read(BlobReadFailure::RecordStream(cause)))?;
        if count == 0 {
            break;
        }
        if bytes.len() + count > MAX_SELECTED_CONTROL_BYTES as usize {
            return Err(BlobReadOpenFailure::PublicationDamaged(
                BlobRecordDenial::Truncated,
            ));
        }
        bytes.extend_from_slice(&scratch[..count]);
    }
    BlobGenerationPublicationV1::decode(&bytes).map_err(BlobReadOpenFailure::PublicationDamaged)
}

/// A negative result is possible only after the entire selected root was
/// inspected. This prevents issuance from treating a truncated scan as proof
/// that a randomly generated object/session identity is unused.
pub(in crate::physical_runtime) fn selected_blob_identity_exists(
    runtime: &ServingPhysicalRuntime,
    identity: [u8; 16],
    max_records: NonZeroU64,
) -> Result<bool, BlobReadOpenFailure> {
    let mut found = false;
    let _ = walk_selected(runtime, max_records, |payload, store| {
        if is_control_kind(payload, 1) {
            let declaration = BlobSessionDeclarationV1::decode(payload)
                .map_err(BlobReadOpenFailure::PublicationDamaged)?;
            if declaration.store() != store.bytes() {
                return Err(BlobReadOpenFailure::ForeignStore);
            }
            found |= declaration.object() == identity || declaration.session() == identity;
        } else if is_control_kind(payload, 4) {
            let publication = BlobGenerationPublicationV1::decode(payload)
                .map_err(BlobReadOpenFailure::PublicationDamaged)?;
            if publication.store() != store.bytes() {
                return Err(BlobReadOpenFailure::ForeignStore);
            }
            found |= publication.object() == identity || publication.session() == identity;
        }
        Ok(())
    })?;
    Ok(found)
}

fn is_control_kind(payload: &[u8], kind: u8) -> bool {
    payload.len() >= 9 && &payload[..8] == BLOB_MAGIC && payload[8] == kind
}

fn walk_selected<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    max_records: NonZeroU64,
    mut visit: impl FnMut(
        &[u8],
        worth_store_physical_format::store_namespace::StableStoreIdentity,
    ) -> Result<(), BlobReadOpenFailure>,
) -> Result<(PhysicalRecordReader, BlobPhysicalAllocation<'runtime>), BlobReadOpenFailure> {
    let allocation = runtime
        .physical_allocations()
        .admit_blob(NonZeroU64::new(READ_CHARGE_BYTES).expect("positive read charge"))
        .map_err(BlobReadOpenFailure::Allocation)?;
    let mut scratch = Vec::new();
    scratch
        .try_reserve_exact(SCAN_SCRATCH_BYTES)
        .map_err(|_| BlobReadOpenFailure::ScratchUnavailable)?;
    scratch.resize(SCAN_SCRATCH_BYTES, 0);
    let reader = runtime
        .records()
        .map_err(BlobReadOpenFailure::RootProtection)?;
    let mut scan = reader
        .scan_rebuild(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(1).expect("one is a nonzero scan batch"))
                .with_payload_limit(
                    crate::physical_runtime::RecordByteLimit::new(
                        MAX_SELECTED_CONTROL_BYTES as u32,
                    )
                    .expect("control frame size is nonzero"),
                ),
        )
        .map_err(BlobReadOpenFailure::Scan)?;
    let mut examined = 0_u64;
    loop {
        if examined == max_records.get() {
            return Err(BlobReadOpenFailure::ScanBoundExhausted);
        }
        match scan
            .read_next_into(&mut scratch)
            .map_err(BlobReadOpenFailure::Scan)?
        {
            RecordScanOutcome::Completed(_) => break,
            RecordScanOutcome::Batch(batch) => {
                for index in 0..batch.records().len() {
                    examined += 1;
                    let record = &batch.records()[index];
                    if let Some(payload) = batch.payload(index) {
                        visit(payload, scan.store_identity())?;
                    } else if record.declared_payload_bytes() <= MAX_SELECTED_CONTROL_BYTES {
                        return Err(BlobReadOpenFailure::PublicationDamaged(
                            BlobRecordDenial::Truncated,
                        ));
                    }
                }
                if batch.is_complete() {
                    break;
                }
            }
        }
    }
    Ok((scan.into_reader(), allocation))
}
