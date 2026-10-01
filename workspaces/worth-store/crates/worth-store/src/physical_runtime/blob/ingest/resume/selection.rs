use std::mem::size_of;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordDenial, BlobRecordV1, BlobSessionDeclarationV1,
    PersistedRecordIdentity,
};

use crate::physical_runtime::{
    durability::CompletedDurableCheckpointWitness, BlobResidentComponent, PhysicalRecordReader,
    RecordByteLimit, RecordCountLimit, RecordScanOutcome, RecordScanRequest,
};

use super::super::super::{AdmittedBlobScope, BlobIngestAllocation, BlobIngestFailure};
use super::super::selected_session::{
    read_authenticated_declaration, selected_abandonment_matches, SelectedSessionFailure,
    DECLARATION_FRAME_BYTES,
};
use super::{
    claims::{validate_selected_claims, SelectedResumeClaim},
    record_read::{has_oversized_blob_prefix, persisted, read_record_into, RESUME_FRAME_BYTES},
    BlobResumeFailure, BlobResumeLimits, BlobResumeObservation, BlobResumeToken,
};

const BLOB_MAGIC: &[u8; 8] = b"WRC11BLB";

pub(super) struct SelectedResumeState {
    pub(super) reader: PhysicalRecordReader,
    pub(super) declaration: BlobSessionDeclarationV1,
    pub(super) claims: Vec<SelectedResumeClaim>,
    pub(super) scratch: Vec<u8>,
    pub(super) observation: BlobResumeObservation,
}

pub(super) fn select(
    reader: PhysicalRecordReader,
    token: BlobResumeToken,
    scope: &AdmittedBlobScope,
    limits: BlobResumeLimits,
    allocation: &mut BlobIngestAllocation<'_>,
    completed: Option<CompletedDurableCheckpointWitness>,
) -> Result<SelectedResumeState, BlobResumeFailure> {
    let reader = reader.into_rebuild();
    let mut scratch = charged_scratch(allocation)?;
    let declaration =
        read_authenticated_declaration(&reader, &token, scope).map_err(map_selected_session)?;
    if allocation.observation().ceiling() > declaration.memory_limit() {
        return Err(BlobResumeFailure::DeclarationMismatch);
    }
    let mut claims = charged_claims(limits, allocation)?;
    let mut observation = BlobResumeObservation {
        selected_payload_bytes: DECLARATION_FRAME_BYTES as u64,
        metadata_capacity_bytes: u64::try_from(claims.capacity())
            .ok()
            .and_then(|capacity| capacity.checked_mul(size_of::<SelectedResumeClaim>() as u64))
            .ok_or(BlobResumeFailure::MetadataCapacity)?,
        ..BlobResumeObservation::default()
    };
    let mut found_declaration = false;
    let mut scan = reader
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(1).expect("one scan row"))
                .with_payload_limit(
                    RecordByteLimit::new(RESUME_FRAME_BYTES as u32).expect("bounded frame limit"),
                ),
        )
        .map_err(BlobResumeFailure::Scan)?;
    loop {
        if observation.scanned_records == limits.maximum_scanned_records().get() {
            return Err(BlobResumeFailure::ScanBoundExhausted);
        }
        let mut deferred = None;
        let complete = match scan
            .read_next_into(&mut scratch)
            .map_err(BlobResumeFailure::Scan)?
        {
            RecordScanOutcome::Completed(_) => break,
            RecordScanOutcome::Batch(batch) => {
                for (index, row) in batch.records().iter().enumerate() {
                    observation.scanned_records += 1;
                    let record = persisted(row.record_id())?;
                    if let Some(payload) = batch.payload(index) {
                        observation.selected_payload_bytes = observation
                            .selected_payload_bytes
                            .saturating_add(payload.len() as u64);
                        observe_selected_payload(
                            payload,
                            record,
                            token,
                            declaration,
                            completed,
                            &mut found_declaration,
                            &mut claims,
                        )?;
                    } else {
                        deferred = Some((record, row.declared_payload_bytes()));
                    }
                }
                batch.is_complete()
            }
        };
        if let Some((record, declared_bytes)) = deferred {
            if declared_bytes > RESUME_FRAME_BYTES as u64 {
                let (reserved_blob, prefix_bytes) =
                    has_oversized_blob_prefix(scan.protected_reader(), record, declared_bytes)?;
                observation.selected_payload_bytes = observation
                    .selected_payload_bytes
                    .saturating_add(prefix_bytes as u64);
                if reserved_blob {
                    return Err(BlobResumeFailure::Format(BlobRecordDenial::FrameTooLarge));
                }
            } else {
                let used = read_record_into(
                    scan.protected_reader(),
                    record,
                    &mut scratch,
                    RESUME_FRAME_BYTES,
                )?;
                observation.selected_payload_bytes = observation
                    .selected_payload_bytes
                    .saturating_add(used as u64);
                observe_selected_payload(
                    &scratch[..used],
                    record,
                    token,
                    declaration,
                    completed,
                    &mut found_declaration,
                    &mut claims,
                )?;
            }
        }
        if complete {
            break;
        }
    }
    if !found_declaration {
        return Err(BlobResumeFailure::DeclarationMismatch);
    }
    validate_selected_claims(&mut claims, declaration)?;
    Ok(SelectedResumeState {
        reader: scan.into_protected_reader(),
        declaration,
        claims,
        scratch,
        observation,
    })
}

fn charged_scratch(
    allocation: &mut BlobIngestAllocation<'_>,
) -> Result<Vec<u8>, BlobResumeFailure> {
    allocation
        .set_live(
            BlobResidentComponent::ResumeScratch,
            RESUME_FRAME_BYTES as u64,
        )
        .map_err(|cause| BlobResumeFailure::Ingest(BlobIngestFailure::Memory(cause)))?;
    let mut scratch = Vec::new();
    scratch
        .try_reserve_exact(RESUME_FRAME_BYTES)
        .map_err(|_| BlobResumeFailure::ScratchUnavailable)?;
    let actual =
        u64::try_from(scratch.capacity()).map_err(|_| BlobResumeFailure::ScratchUnavailable)?;
    allocation
        .set_live(BlobResidentComponent::ResumeScratch, actual)
        .map_err(|cause| BlobResumeFailure::Ingest(BlobIngestFailure::Memory(cause)))?;
    scratch.resize(RESUME_FRAME_BYTES, 0);
    Ok(scratch)
}

fn charged_claims(
    limits: BlobResumeLimits,
    allocation: &mut BlobIngestAllocation<'_>,
) -> Result<Vec<SelectedResumeClaim>, BlobResumeFailure> {
    let bytes = limits.metadata_bytes().get();
    let element_bytes = size_of::<SelectedResumeClaim>() as u64;
    let slots =
        usize::try_from(bytes / element_bytes).map_err(|_| BlobResumeFailure::MetadataCapacity)?;
    if slots == 0 {
        return Err(BlobResumeFailure::MetadataCapacity);
    }
    allocation
        .set_live(
            BlobResidentComponent::ResumeMetadata,
            slots as u64 * element_bytes,
        )
        .map_err(|cause| BlobResumeFailure::Ingest(BlobIngestFailure::Memory(cause)))?;
    let mut claims = Vec::new();
    claims
        .try_reserve_exact(slots)
        .map_err(|_| BlobResumeFailure::MetadataCapacity)?;
    let actual = u64::try_from(claims.capacity())
        .ok()
        .and_then(|capacity| capacity.checked_mul(element_bytes))
        .ok_or(BlobResumeFailure::MetadataCapacity)?;
    if actual > bytes {
        return Err(BlobResumeFailure::MetadataCapacity);
    }
    allocation
        .set_live(BlobResidentComponent::ResumeMetadata, actual)
        .map_err(|cause| BlobResumeFailure::Ingest(BlobIngestFailure::Memory(cause)))?;
    Ok(claims)
}

fn map_selected_session(cause: SelectedSessionFailure) -> BlobResumeFailure {
    match cause {
        SelectedSessionFailure::Format(cause) => BlobResumeFailure::Format(cause),
        SelectedSessionFailure::Read(cause) => BlobResumeFailure::Read(cause),
        SelectedSessionFailure::Stream(cause) => BlobResumeFailure::Stream(cause),
        SelectedSessionFailure::ForeignStore => BlobResumeFailure::ForeignStore,
        SelectedSessionFailure::DeclarationMismatch => BlobResumeFailure::DeclarationMismatch,
        SelectedSessionFailure::ScopeMismatch => BlobResumeFailure::ScopeMismatch,
    }
}

fn observe_selected_payload(
    payload: &[u8],
    record: PersistedRecordIdentity,
    token: BlobResumeToken,
    declaration: BlobSessionDeclarationV1,
    completed: Option<CompletedDurableCheckpointWitness>,
    found_declaration: &mut bool,
    claims: &mut Vec<SelectedResumeClaim>,
) -> Result<(), BlobResumeFailure> {
    if !payload.starts_with(BLOB_MAGIC) {
        return Ok(());
    }
    let decoded = decode_blob_record(payload).map_err(BlobResumeFailure::Format)?;
    let claim = match decoded {
        BlobRecordV1::DropSetManifest(_)
        | BlobRecordV1::DropSetManifestV2(_)
        | BlobRecordV1::DropSetManifestV3(_)
        | BlobRecordV1::OriginalDropReserved(_)
        | BlobRecordV1::ReclaimDescriptor(_)
        | BlobRecordV1::ReclaimDescriptorV2(_)
        | BlobRecordV1::ReclaimDescriptorV3(_)
        | BlobRecordV1::DedupeQuarantine(_) => None,
        value @ (BlobRecordV1::ChunkReuseClaim(_) | BlobRecordV1::ChunkReuseClaimV2(_)) => {
            let reuse = match value {
                BlobRecordV1::ChunkReuseClaim(value) => value,
                BlobRecordV1::ChunkReuseClaimV2(value) => value.claim(),
                _ => unreachable!(),
            };
            (reuse.destination_session() == token.session)
                .then(|| {
                    if reuse.store() != token.store
                        || reuse.scope() != declaration.key_scope()
                        || reuse.chunk_size() != token.chunk_size
                    {
                        return Err(BlobResumeFailure::ConflictingClaims);
                    }
                    Ok(SelectedResumeClaim::ReusedChunk {
                        ordinal: reuse.destination_ordinal(),
                        record,
                        digest: reuse.stored_digest(),
                        bytes: u64::from(reuse.chunk_length()),
                    })
                })
                .transpose()?
        }
        BlobRecordV1::SessionDeclared(selected) => {
            if selected.session() == token.session {
                if *found_declaration
                    || record != token.declaration_record
                    || selected != declaration
                {
                    return Err(BlobResumeFailure::ConflictingClaims);
                }
                *found_declaration = true;
            } else if selected.object() == declaration.object() {
                return Err(BlobResumeFailure::ConflictingClaims);
            }
            None
        }
        BlobRecordV1::Chunk(chunk) => {
            let occurrence = chunk.occurrence();
            (occurrence.session() == token.session)
                .then(|| {
                    if occurrence.store() != token.store || chunk.chunk_size() != token.chunk_size {
                        return Err(BlobResumeFailure::ConflictingClaims);
                    }
                    Ok(SelectedResumeClaim::Chunk {
                        ordinal: occurrence.ordinal(),
                        record,
                        digest: chunk.stored_digest(),
                        bytes: chunk.bytes().len() as u64,
                    })
                })
                .transpose()?
        }
        BlobRecordV1::TreeNode(node) => {
            let occurrence = node.occurrence();
            (occurrence.session() == token.session)
                .then(|| {
                    if occurrence.store() != token.store || occurrence.index() >= (1_u64 << 56) {
                        return Err(BlobResumeFailure::TreeConflict);
                    }
                    Ok(SelectedResumeClaim::Node {
                        ordinal: (u64::from(occurrence.level()) << 56) | occurrence.index(),
                        record,
                        frame_digest: Sha256::digest(payload).into(),
                    })
                })
                .transpose()?
        }
        BlobRecordV1::SessionFrontier(frontier) => (frontier.session() == token.session
            || frontier.declaration_record() == token.declaration_record)
            .then(|| {
                if frontier.store() != token.store
                    || frontier.session() != token.session
                    || frontier.declaration_record() != token.declaration_record
                    || frontier.declaration_digest() != token.declaration_digest
                {
                    return Err(BlobResumeFailure::ConflictingClaims);
                }
                Ok(SelectedResumeClaim::Frontier {
                    ordinal: frontier.next_chunk_ordinal(),
                    durable_bytes: frontier.durable_bytes(),
                    last_record: frontier.last_chunk_record(),
                    last_digest: frontier.last_chunk_digest(),
                })
            })
            .transpose()?,
        BlobRecordV1::GenerationPublished(publication) => {
            if publication.object() == declaration.object()
                || publication.session() == token.session
            {
                return Err(BlobResumeFailure::AlreadyPublished);
            }
            None
        }
        BlobRecordV1::SessionAbandoned(abandoned) => {
            if abandoned.session() == token.session
                || abandoned.declaration_record() == token.declaration_record
            {
                if !selected_abandonment_matches(abandoned, &token, declaration, completed) {
                    return Err(BlobResumeFailure::ConflictingClaims);
                }
                return Err(BlobResumeFailure::AlreadyAbandoned);
            }
            None
        }
    };
    if let Some(claim) = claim {
        if claims.len() == claims.capacity() {
            return Err(BlobResumeFailure::MetadataCapacity);
        }
        claims.push(claim);
    }
    Ok(())
}

#[cfg(test)]
#[path = "selection/terminal_tests.rs"]
mod terminal_tests;
