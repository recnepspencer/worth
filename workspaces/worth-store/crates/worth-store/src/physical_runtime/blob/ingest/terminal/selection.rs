use worth_store_physical_format::{
    decode_blob_record, BlobRecordDenial, BlobRecordKind, BlobRecordV1, BlobSessionDeclarationV1,
    DropSetManifestV3View, PersistedRecordIdentity,
};

use crate::physical_runtime::{
    durability::CompletedDurableCheckpointWitness, PhysicalRecordId, PhysicalRecordReader,
    RecordByteLimit, RecordCountLimit, RecordReadLimits, RecordScanOutcome, RecordScanRequest,
};

use super::super::{
    resume::BlobResumeToken,
    selected_session::{
        read_authenticated_declaration, release_source_names_session, selected_abandonment_matches,
    },
};
use super::{contracts::BlobTerminalLimits, BlobTerminalFailure};

/// Covers every frame whose fields can decide the abort's fate; the largest
/// is a V2 chunk-reuse claim (48-byte header, 436-byte payload). A V3
/// drop-set manifest may be larger, and decides by its leading window alone.
const CONTROL_SCAN_BYTES: usize = 512;
const BLOB_MAGIC: &[u8; 8] = b"WRC11BLB";
/// The magic and the kind byte of a deferred frame.
const KIND_PREFIX_BYTES: usize = 9;
const MANIFEST_V3: u8 = BlobRecordKind::DropSetManifestV3 as u8;
const MANIFEST_V3_WINDOW: usize = DropSetManifestV3View::SOURCE_BASIS_PREFIX_BYTES;
/// The kinds whose fields cannot decide the fate: a frame of one of them may
/// exceed the control scan unread.
const UNREAD_WHEN_DEFERRED: &[BlobRecordKind] = &[
    BlobRecordKind::Chunk,
    BlobRecordKind::TreeNode,
    BlobRecordKind::DropSetManifest,
    BlobRecordKind::ReclaimDescriptor,
    BlobRecordKind::DropSetManifestV2,
    BlobRecordKind::OriginalDropReserved,
    BlobRecordKind::DedupeQuarantine,
    BlobRecordKind::ReclaimDescriptorV2,
    BlobRecordKind::ReclaimDescriptorV3,
];
// Every deferred frame is longer than the control scan, so a deferred V3
// manifest always holds its whole source window.
const _: () = assert!(KIND_PREFIX_BYTES <= MANIFEST_V3_WINDOW);
const _: () = assert!(MANIFEST_V3_WINDOW <= CONTROL_SCAN_BYTES);

pub(super) struct SelectedTerminalState {
    pub(super) _reader: PhysicalRecordReader,
    pub(super) existing: Option<PersistedRecordIdentity>,
    pub(super) maximum_checkpoint_sequence: u64,
}

pub(super) fn select(
    reader: PhysicalRecordReader,
    token: BlobResumeToken,
    scope: &crate::physical_runtime::AdmittedBlobScope,
    limits: BlobTerminalLimits,
    completed: Option<CompletedDurableCheckpointWitness>,
) -> Result<SelectedTerminalState, BlobTerminalFailure> {
    let reader = reader.into_rebuild();
    let declaration = read_authenticated_declaration(&reader, &token, scope)
        .map_err(BlobTerminalFailure::from)?;
    let mut found_declaration = false;
    let mut existing = None;
    let mut examined = 0;
    let mut scratch = [0_u8; CONTROL_SCAN_BYTES];
    let mut scan = reader
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(1).expect("one scan row"))
                .with_payload_limit(
                    RecordByteLimit::new(CONTROL_SCAN_BYTES as u32).expect("bounded control scan"),
                ),
        )
        .map_err(BlobTerminalFailure::Scan)?;
    loop {
        if examined == limits.maximum_scanned_records().get() {
            return Err(BlobTerminalFailure::ScanBoundExhausted);
        }
        let mut deferred = None;
        let complete = match scan
            .read_next_into(&mut scratch)
            .map_err(BlobTerminalFailure::Scan)?
        {
            RecordScanOutcome::Completed(_) => break,
            RecordScanOutcome::Batch(batch) => {
                for (index, row) in batch.records().iter().enumerate() {
                    examined += 1;
                    let record = PersistedRecordIdentity::new(
                        row.record_id().allocation_epoch(),
                        row.record_id().ordinal(),
                    )
                    .ok_or(BlobTerminalFailure::Format(
                        BlobRecordDenial::InvalidIdentity,
                    ))?;
                    if let Some(payload) = batch.payload(index) {
                        inspect_control(
                            payload,
                            record,
                            token,
                            declaration,
                            completed,
                            &mut found_declaration,
                            &mut existing,
                        )?;
                    } else {
                        deferred = Some((record, row.declared_payload_bytes()));
                    }
                }
                batch.is_complete()
            }
        };
        if let Some((record, declared_bytes)) = deferred {
            inspect_deferred(
                scan.protected_reader(),
                record,
                declared_bytes,
                &token,
                declaration,
            )?;
        }
        if complete {
            break;
        }
    }
    if !found_declaration {
        return Err(BlobTerminalFailure::DeclarationMismatch);
    }
    Ok(SelectedTerminalState {
        _reader: scan.into_protected_reader(),
        existing,
        maximum_checkpoint_sequence: declaration.max_checkpoint_sequence(),
    })
}

fn inspect_control(
    payload: &[u8],
    record: PersistedRecordIdentity,
    token: BlobResumeToken,
    declaration: BlobSessionDeclarationV1,
    completed: Option<CompletedDurableCheckpointWitness>,
    found_declaration: &mut bool,
    existing: &mut Option<PersistedRecordIdentity>,
) -> Result<(), BlobTerminalFailure> {
    if !payload.starts_with(BLOB_MAGIC) {
        return Ok(());
    }
    match decode_blob_record(payload).map_err(BlobTerminalFailure::Format)? {
        BlobRecordV1::SessionDeclared(selected) => {
            if selected.session() == token.session {
                if *found_declaration
                    || record != token.declaration_record
                    || selected != declaration
                {
                    return Err(BlobTerminalFailure::ConflictingSelectedFate);
                }
                *found_declaration = true;
            } else if selected.object() == declaration.object() {
                return Err(BlobTerminalFailure::ConflictingSelectedFate);
            }
        }
        BlobRecordV1::GenerationPublished(published) => {
            if published.session() == token.session || published.object() == declaration.object() {
                return Err(BlobTerminalFailure::AlreadyPublished);
            }
        }
        BlobRecordV1::SessionAbandoned(abandoned) => {
            if abandoned.session() == token.session
                || abandoned.declaration_record() == token.declaration_record
            {
                if !selected_abandonment_matches(abandoned, &token, declaration, completed)
                    || existing.replace(record).is_some()
                {
                    return Err(BlobTerminalFailure::ConflictingSelectedFate);
                }
            }
        }
        BlobRecordV1::SessionFrontier(frontier) => {
            if (frontier.session() == token.session
                || frontier.declaration_record() == token.declaration_record)
                && (frontier.store() != token.store
                    || frontier.session() != token.session
                    || frontier.declaration_record() != token.declaration_record
                    || frontier.declaration_digest() != token.declaration_digest)
            {
                return Err(BlobTerminalFailure::ConflictingSelectedFate);
            }
        }
        BlobRecordV1::DropSetManifestV3(manifest) => {
            if release_source_names_session(manifest.source_basis(), &token, declaration) {
                return Err(BlobTerminalFailure::AlreadyReleased);
            }
        }
        BlobRecordV1::Chunk(_)
        | BlobRecordV1::TreeNode(_)
        | BlobRecordV1::DropSetManifest(_)
        | BlobRecordV1::DropSetManifestV2(_)
        | BlobRecordV1::OriginalDropReserved(_)
        | BlobRecordV1::ReclaimDescriptor(_)
        | BlobRecordV1::ReclaimDescriptorV2(_)
        | BlobRecordV1::ReclaimDescriptorV3(_)
        | BlobRecordV1::DedupeQuarantine(_) => {}
        value @ (BlobRecordV1::ChunkReuseClaim(_) | BlobRecordV1::ChunkReuseClaimV2(_)) => {
            let reuse = match value {
                BlobRecordV1::ChunkReuseClaim(value) => value,
                BlobRecordV1::ChunkReuseClaimV2(value) => value.claim(),
                _ => unreachable!(),
            };
            if reuse.destination_session() == token.session
                && (reuse.store() != token.store
                    || reuse.scope() != declaration.key_scope()
                    || reuse.chunk_size() != declaration.chunk_size()
                    || reuse
                        .destination_ordinal()
                        .checked_mul(u64::from(declaration.chunk_size()))
                        .and_then(|start| declaration.declared_bytes().checked_sub(start))
                        .filter(|bytes| *bytes > 0)
                        .map(|remaining| remaining.min(u64::from(declaration.chunk_size())))
                        != Some(u64::from(reuse.chunk_length())))
            {
                return Err(BlobTerminalFailure::ConflictingSelectedFate);
            }
        }
    }
    Ok(())
}

/// Large selected frames are not rehashed merely to abort: only the leading
/// bytes that can decide the fate are read.
fn inspect_deferred(
    reader: &PhysicalRecordReader,
    record: PersistedRecordIdentity,
    declared_bytes: u64,
    token: &BlobResumeToken,
    declaration: BlobSessionDeclarationV1,
) -> Result<(), BlobTerminalFailure> {
    let limit = u32::try_from(declared_bytes)
        .ok()
        .and_then(RecordByteLimit::new)
        .ok_or(BlobTerminalFailure::Format(BlobRecordDenial::FrameTooLarge))?;
    let mut stream = reader
        .open(
            PhysicalRecordId::from_persisted(record),
            RecordReadLimits::new(limit),
        )
        .map_err(BlobTerminalFailure::Read)?;
    let mut prefix = [0_u8; MANIFEST_V3_WINDOW];
    let mut wanted = KIND_PREFIX_BYTES;
    let mut used = 0;
    while used < wanted {
        let count = stream
            .read_next(&mut prefix[used..wanted])
            .map_err(BlobTerminalFailure::Stream)?;
        if count == 0 {
            return Err(BlobTerminalFailure::Format(BlobRecordDenial::Truncated));
        }
        used += count;
        if used == KIND_PREFIX_BYTES && prefix.starts_with(BLOB_MAGIC) && prefix[8] == MANIFEST_V3 {
            wanted = MANIFEST_V3_WINDOW;
        }
    }
    deferred_fate(&prefix[..used], token, declaration)
}

/// A V3 drop-set manifest decides by the source its leading window names.
/// Only the kinds in `UNREAD_WHEN_DEFERRED` stay unread; a deferred frame of
/// any other kind cannot be silently mistaken for unrelated data.
fn deferred_fate(
    prefix: &[u8],
    token: &BlobResumeToken,
    declaration: BlobSessionDeclarationV1,
) -> Result<(), BlobTerminalFailure> {
    if !prefix.starts_with(BLOB_MAGIC) {
        return Ok(());
    }
    let kind = prefix[8];
    if kind == MANIFEST_V3 {
        let source = DropSetManifestV3View::source_basis_in_prefix(prefix)
            .map_err(BlobTerminalFailure::Format)?;
        if release_source_names_session(source, token, declaration) {
            return Err(BlobTerminalFailure::AlreadyReleased);
        }
        return Ok(());
    }
    if UNREAD_WHEN_DEFERRED
        .iter()
        .any(|unread| *unread as u8 == kind)
    {
        return Ok(());
    }
    Err(BlobTerminalFailure::Format(BlobRecordDenial::FrameTooLarge))
}

#[cfg(test)]
#[path = "selection/tests.rs"]
mod tests;
