use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{BlobRecordDenial, PersistedRecordIdentity};

use crate::physical_runtime::blob::BlobIngestAllocation;
use crate::physical_runtime::{
    layout::{PhysicalLayoutAccess, PhysicalLayoutDenial},
    BlobMemoryDenial, BlobResidentComponent, PhysicalReadProtectionDenial, RecordReadError,
    RecordStreamFailure, ServingPhysicalRuntime,
};

use super::{
    key::{DedupeIndexKey, DedupeIndexValue},
    source::{verify_source, VerifiedDedupeSource},
};

#[derive(Debug)]
pub enum BlobDedupeFailure {
    Memory(BlobMemoryDenial),
    RootProtection(PhysicalReadProtectionDenial),
    Layout(PhysicalLayoutDenial),
    InvalidDerivedValue,
    SourceRead(RecordReadError),
    SourceStream(RecordStreamFailure),
    SourceFormat(BlobRecordDenial),
    SourceScopeMismatch,
    SourceTreeDamaged,
    ScratchUnavailable,
    DigestCollisionDenied {
        scope: [u8; 32],
        digest: [u8; 32],
        source_publication: PersistedRecordIdentity,
        source_ordinal: u64,
        source_chunk: PersistedRecordIdentity,
    },
}

pub(in crate::physical_runtime::blob) fn lookup_reusable_chunk(
    runtime: &ServingPhysicalRuntime,
    allocation: &mut BlobIngestAllocation<'_>,
    scope: [u8; 32],
    chunk_size: u32,
    bytes: &[u8],
    forced_digest: Option<[u8; 32]>,
) -> Result<Option<VerifiedDedupeSource>, BlobDedupeFailure> {
    allocation
        .set_live(BlobResidentComponent::Scratch, 512 * 1024)
        .map_err(BlobDedupeFailure::Memory)?;
    let result = lookup_charged(runtime, scope, chunk_size, bytes, forced_digest);
    allocation
        .set_live(BlobResidentComponent::Scratch, 0)
        .map_err(BlobDedupeFailure::Memory)?;
    result
}

fn lookup_charged(
    runtime: &ServingPhysicalRuntime,
    scope: [u8; 32],
    chunk_size: u32,
    bytes: &[u8],
    forced_digest: Option<[u8; 32]>,
) -> Result<Option<VerifiedDedupeSource>, BlobDedupeFailure> {
    let key = forced_digest
        .map(|digest| DedupeIndexKey::from_digest(scope, digest))
        .unwrap_or_else(|| DedupeIndexKey::for_chunk(scope, chunk_size, bytes));
    let reader = runtime
        .records()
        .map_err(BlobDedupeFailure::RootProtection)?;
    let layouts =
        PhysicalLayoutAccess::from_reader(runtime, reader).map_err(BlobDedupeFailure::Layout)?;
    let found = layouts
        .btree(DurableArtifactFamilyId::DedupeIndex)
        .and_then(|tree| tree.point_raw(&key.bytes()))
        .map_err(BlobDedupeFailure::Layout)?
        .0;
    let Some(value) = found else { return Ok(None) };
    let locator = DedupeIndexValue::decode(&value).ok_or(BlobDedupeFailure::InvalidDerivedValue)?;
    let reader = layouts.into_reader();
    verify_source(&reader, locator, scope, key.digest(), bytes, chunk_size).map(Some)
}
