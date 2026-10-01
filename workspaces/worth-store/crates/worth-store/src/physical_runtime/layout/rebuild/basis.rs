use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

mod deferred;
use deferred::admit_deferred_blob;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobChunkReuseClaimV1, BlobChunkReuseClaimV2, BlobDedupeQuarantineV1,
    BlobGenerationPublicationV1, BlobRecordDenial, BlobRecordV1, BlobSessionDeclarationV1,
    IndexedThroughBlobPublication, PersistedRecordIdentity, BLOB_CONTROL_FRAME_MAX_BYTES,
};

use crate::physical_runtime::{
    blob::{verify_selected_quarantine, BlobDedupeFailure, DedupeIndexKey},
    layout::{
        PhysicalIndexPointKeyDenial, PhysicalLayoutAppendFailure, PhysicalLayoutDenial,
        PhysicalLayoutMaintenanceFailure, PhysicalLayoutPageReadFailure,
    },
    MaintenancePhysicalAllocation, PhysicalReadProtectionDenial, PhysicalRecordReader,
    PhysicalScopedAllocationFailure, RecordByteLimit, RecordCountLimit, RecordReadError,
    RecordScanError, RecordScanOutcome, RecordScanRequest, RecordStreamFailure,
    ServingPhysicalRuntime,
};

/// Explicit reconstruction envelope. The Store, not an external row provider,
/// selects the authority source under one protected C.5 root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutRebuildLimits {
    maximum_selected_records: NonZeroU64,
    maximum_control_records: NonZeroU64,
}

impl LayoutRebuildLimits {
    pub const fn new(
        maximum_selected_records: NonZeroU64,
        maximum_control_records: NonZeroU64,
    ) -> Self {
        Self {
            maximum_selected_records,
            maximum_control_records,
        }
    }

    pub const fn maximum_selected_records(self) -> u64 {
        self.maximum_selected_records.get()
    }
    pub const fn maximum_control_records(self) -> u64 {
        self.maximum_control_records.get()
    }
}

#[derive(Debug)]
pub enum LayoutRebuildFailure {
    RootProtection(PhysicalReadProtectionDenial),
    Allocation(PhysicalScopedAllocationFailure),
    Scan(RecordScanError),
    SelectedBoundExhausted,
    ControlBoundExhausted,
    ScratchUnavailable,
    InvalidSelectedIdentity,
    MalformedSelectedBlob(BlobRecordDenial),
    ForeignStore,
    ConflictingSession,
    ConflictingReuseOrdinal,
    QuarantineAuthorityDamaged,
    PublicationWithoutDeclaration,
    LatestPublicationMissing,
    AuthorityRead(RecordReadError),
    AuthorityStream(RecordStreamFailure),
    TreeDamaged,
    ChunkDamaged,
    ReuseAuthority(BlobDedupeFailure),
    ReuseSourceNotSelected,
    ReuseClaimNotSelected,
    LogicalDigestMismatch,
    TraversalBoundExhausted,
    DeferredBlobAuthority,
    NoSelectedPublication,
    ConcurrentAuthorityAdvance,
    InvalidCatalogKey(PhysicalIndexPointKeyDenial),
    LayoutRead(PhysicalLayoutPageReadFailure),
    LayoutPoint(PhysicalLayoutDenial),
    LayoutMutation(PhysicalLayoutMaintenanceFailure),
    InvalidDerivedLocator,
    DirectoryFormat(worth_store_physical_format::DerivedFamilyDirectoryDenial),
    DirectoryAppend(PhysicalLayoutAppendFailure),
    UnregisteredFamily(worth_store_contracts::DurableArtifactFamilyId),
}

#[derive(Debug, Clone, Copy)]
pub(in crate::physical_runtime) struct SelectedBlobPublication {
    pub(in crate::physical_runtime) record: PersistedRecordIdentity,
    pub(in crate::physical_runtime) frame_digest: [u8; 32],
    pub(in crate::physical_runtime) publication: BlobGenerationPublicationV1,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::physical_runtime) struct SelectedReuseClaim {
    pub(in crate::physical_runtime) record: PersistedRecordIdentity,
    pub(in crate::physical_runtime) claim: BlobChunkReuseClaimV1,
    pub(in crate::physical_runtime) witness: Option<BlobChunkReuseClaimV2>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::physical_runtime) struct SelectedDedupeQuarantine {
    pub(in crate::physical_runtime) record: PersistedRecordIdentity,
    pub(in crate::physical_runtime) claim: BlobDedupeQuarantineV1,
}

pub(in crate::physical_runtime) struct LayoutRebuildAuthority<'runtime> {
    pub(in crate::physical_runtime) reader: PhysicalRecordReader,
    pub(in crate::physical_runtime) latest: Option<IndexedThroughBlobPublication>,
    pub(in crate::physical_runtime) publications: Vec<SelectedBlobPublication>,
    pub(in crate::physical_runtime) declarations: BTreeMap<[u8; 16], BlobSessionDeclarationV1>,
    pub(in crate::physical_runtime) reuse_claims: BTreeMap<([u8; 16], u64), SelectedReuseClaim>,
    pub(in crate::physical_runtime) quarantined_keys: BTreeSet<[u8; 64]>,
    pub(in crate::physical_runtime) scanned_records: u64,
    _allocation: MaintenancePhysicalAllocation<'runtime>,
}

pub(in crate::physical_runtime) fn collect_selected_blob_authority<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    limits: LayoutRebuildLimits,
) -> Result<LayoutRebuildAuthority<'runtime>, LayoutRebuildFailure> {
    let charged = limits
        .maximum_control_records
        .get()
        .checked_mul(512)
        .and_then(|bytes| {
            limits
                .maximum_selected_records
                .get()
                .checked_mul(160)
                .and_then(|selected| bytes.checked_add(selected))
        })
        .and_then(|bytes| bytes.checked_add(BLOB_CONTROL_FRAME_MAX_BYTES as u64))
        .and_then(NonZeroU64::new)
        .ok_or(LayoutRebuildFailure::ControlBoundExhausted)?;
    let allocation = runtime
        .physical_allocations()
        .admit_maintenance(charged)
        .map_err(LayoutRebuildFailure::Allocation)?;
    let mut scratch = Vec::new();
    scratch
        .try_reserve_exact(BLOB_CONTROL_FRAME_MAX_BYTES)
        .map_err(|_| LayoutRebuildFailure::ScratchUnavailable)?;
    scratch.resize(BLOB_CONTROL_FRAME_MAX_BYTES, 0);
    let reader = runtime
        .records()
        .map_err(LayoutRebuildFailure::RootProtection)?;
    let latest = reader.selected_latest_blob_publication();
    let latest_quarantine = reader.selected_latest_blob_quarantine();
    let mut scan = reader
        .scan_rebuild(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(1).expect("nonzero batch"))
                .with_payload_limit(
                    RecordByteLimit::new(BLOB_CONTROL_FRAME_MAX_BYTES as u32)
                        .expect("nonzero control frame"),
                ),
        )
        .map_err(LayoutRebuildFailure::Scan)?;
    let mut publications = Vec::new();
    let mut declarations = BTreeMap::new();
    let mut reuse_claims = BTreeMap::new();
    let mut quarantines = Vec::new();
    let mut scanned = 0_u64;
    loop {
        if scanned == limits.maximum_selected_records.get() {
            return Err(LayoutRebuildFailure::SelectedBoundExhausted);
        }
        let result = scan
            .read_next_into(&mut scratch)
            .map_err(LayoutRebuildFailure::Scan)?;
        let RecordScanOutcome::Batch(batch) = result else {
            break;
        };
        let mut deferred = None;
        for (index, row) in batch.records().iter().enumerate() {
            scanned += 1;
            let record = PersistedRecordIdentity::new(
                row.record_id().allocation_epoch(),
                row.record_id().ordinal(),
            )
            .ok_or(LayoutRebuildFailure::InvalidSelectedIdentity)?;
            let Some(payload) = batch.payload(index) else {
                deferred = Some((record, row.declared_payload_bytes()));
                continue;
            };
            if !payload.starts_with(b"WRC11BLB") {
                continue;
            }
            let fact =
                decode_blob_record(payload).map_err(LayoutRebuildFailure::MalformedSelectedBlob)?;
            match fact {
                BlobRecordV1::SessionDeclared(value) => {
                    enforce_control_bound(
                        &publications,
                        &declarations,
                        &reuse_claims,
                        &quarantines,
                        limits,
                    )?;
                    if value.store() != runtime.store_identity().bytes() {
                        return Err(LayoutRebuildFailure::ForeignStore);
                    }
                    if declarations.insert(value.session(), value).is_some() {
                        return Err(LayoutRebuildFailure::ConflictingSession);
                    }
                }
                BlobRecordV1::GenerationPublished(value) => {
                    enforce_control_bound(
                        &publications,
                        &declarations,
                        &reuse_claims,
                        &quarantines,
                        limits,
                    )?;
                    if value.store() != runtime.store_identity().bytes() {
                        return Err(LayoutRebuildFailure::ForeignStore);
                    }
                    publications.push(SelectedBlobPublication {
                        record,
                        frame_digest: Sha256::digest(payload).into(),
                        publication: value,
                    });
                }
                BlobRecordV1::ChunkReuseClaim(value) => {
                    enforce_control_bound(
                        &publications,
                        &declarations,
                        &reuse_claims,
                        &quarantines,
                        limits,
                    )?;
                    if value.store() != runtime.store_identity().bytes() {
                        return Err(LayoutRebuildFailure::ForeignStore);
                    }
                    if reuse_claims
                        .insert(
                            (value.destination_session(), value.destination_ordinal()),
                            SelectedReuseClaim {
                                record,
                                claim: value,
                                witness: None,
                            },
                        )
                        .is_some()
                    {
                        return Err(LayoutRebuildFailure::ConflictingReuseOrdinal);
                    }
                }
                BlobRecordV1::ChunkReuseClaimV2(value) => {
                    enforce_control_bound(
                        &publications,
                        &declarations,
                        &reuse_claims,
                        &quarantines,
                        limits,
                    )?;
                    let claim = value.claim();
                    if claim.store() != runtime.store_identity().bytes() {
                        return Err(LayoutRebuildFailure::ForeignStore);
                    }
                    if reuse_claims
                        .insert(
                            (claim.destination_session(), claim.destination_ordinal()),
                            SelectedReuseClaim {
                                record,
                                claim,
                                witness: Some(value),
                            },
                        )
                        .is_some()
                    {
                        return Err(LayoutRebuildFailure::ConflictingSession);
                    }
                }
                BlobRecordV1::DedupeQuarantine(value) => {
                    enforce_control_bound(
                        &publications,
                        &declarations,
                        &reuse_claims,
                        &quarantines,
                        limits,
                    )?;
                    if value.store() != runtime.store_identity().bytes() {
                        return Err(LayoutRebuildFailure::ForeignStore);
                    }
                    quarantines.push(SelectedDedupeQuarantine {
                        record,
                        claim: value,
                    });
                }
                _ => {}
            }
        }
        let complete = batch.is_complete();
        drop(batch);
        if let Some((record, declared_bytes)) = deferred {
            admit_deferred_blob(scan.protected_reader(), record, declared_bytes)?;
        }
        if complete {
            break;
        }
    }
    for selected in &publications {
        let publication = selected.publication;
        let declaration = declarations
            .get(&publication.session())
            .ok_or(LayoutRebuildFailure::PublicationWithoutDeclaration)?;
        if declaration.store() != publication.store()
            || declaration.object() != publication.object()
            || declaration.key_scope() != publication.key_scope()
            || declaration.chunk_size() != publication.chunk_size()
            || declaration.declared_bytes() != publication.total_bytes()
        {
            return Err(LayoutRebuildFailure::PublicationWithoutDeclaration);
        }
    }
    if let Some(latest) = latest {
        if !publications.iter().any(|selected| {
            selected.record == latest.record() && selected.frame_digest == latest.encoded_digest()
        }) {
            return Err(LayoutRebuildFailure::LatestPublicationMissing);
        }
    }
    let mut quarantined_keys = BTreeSet::new();
    for selected in &quarantines {
        let claim = selected.claim;
        let source = publications
            .iter()
            .find(|publication| publication.record == claim.source_publication())
            .ok_or(LayoutRebuildFailure::QuarantineAuthorityDamaged)?;
        let destination = declarations
            .get(&claim.destination_session())
            .ok_or(LayoutRebuildFailure::QuarantineAuthorityDamaged)?;
        if source.publication.key_scope() != claim.scope()
            || source.publication.chunk_size() != claim.chunk_size()
            || destination.key_scope() != claim.scope()
            || destination.chunk_size() != claim.chunk_size()
        {
            return Err(LayoutRebuildFailure::QuarantineAuthorityDamaged);
        }
        let start = claim
            .destination_ordinal()
            .checked_mul(u64::from(claim.chunk_size()))
            .filter(|start| *start < destination.declared_bytes())
            .ok_or(LayoutRebuildFailure::QuarantineAuthorityDamaged)?;
        let expected_length =
            (destination.declared_bytes() - start).min(u64::from(claim.chunk_size()));
        let selected_length = verify_selected_quarantine(scan.protected_reader(), claim)
            .map_err(LayoutRebuildFailure::ReuseAuthority)?;
        if u64::from(selected_length) != expected_length {
            return Err(LayoutRebuildFailure::QuarantineAuthorityDamaged);
        }
        quarantined_keys
            .insert(DedupeIndexKey::from_digest(claim.scope(), claim.disputed_digest()).bytes());
    }
    if latest_quarantine.is_some() != !quarantines.is_empty()
        || latest_quarantine
            .is_some_and(|latest| !quarantines.iter().any(|selected| selected.record == latest))
    {
        return Err(LayoutRebuildFailure::QuarantineAuthorityDamaged);
    }
    Ok(LayoutRebuildAuthority {
        reader: scan.into_protected_reader(),
        latest,
        publications,
        declarations,
        reuse_claims,
        quarantined_keys,
        scanned_records: scanned,
        _allocation: allocation,
    })
}

fn enforce_control_bound(
    publications: &[SelectedBlobPublication],
    declarations: &BTreeMap<[u8; 16], BlobSessionDeclarationV1>,
    reuse_claims: &BTreeMap<([u8; 16], u64), SelectedReuseClaim>,
    quarantines: &[SelectedDedupeQuarantine],
    limits: LayoutRebuildLimits,
) -> Result<(), LayoutRebuildFailure> {
    let count = publications
        .len()
        .checked_add(declarations.len())
        .and_then(|count| count.checked_add(reuse_claims.len()))
        .and_then(|count| count.checked_add(quarantines.len()))
        .ok_or(LayoutRebuildFailure::ControlBoundExhausted)?;
    if count as u64 >= limits.maximum_control_records.get() {
        return Err(LayoutRebuildFailure::ControlBoundExhausted);
    }
    Ok(())
}
