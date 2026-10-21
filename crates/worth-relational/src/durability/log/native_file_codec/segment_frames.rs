//! Current native segment framing. Envelope bytes are prepared before movement;
//! settlement writes the real assigned position without decoding prior payloads.
use super::*;
use crate::durability::data::RelationalRecoveryStop;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;
use std::io::{Read, Seek, SeekFrom, Write};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
const MAGIC: &[u8] = b"WORTH-REL-SEG6\n";
const HEADER: usize = 24;
fn corrupt(detail: &str) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptSegment, detail)
}
fn open(path: &Path) -> Result<fs::File, DurabilityError> {
    let mut input = fs::File::open(path).map_err(super::super::local_store::io_error)?;
    let mut magic = [0u8; 15];
    input
        .read_exact(&mut magic[..MAGIC.len()])
        .map_err(|_| corrupt("truncated segment format"))?;
    if &magic[..MAGIC.len()] != MAGIC {
        return Err(DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            "unsupported native segment format; current segments require SEG6",
        ));
    }
    Ok(input)
}
fn header(input: &mut fs::File) -> Result<Option<(u64, u64, u64)>, DurabilityError> {
    let offset = input
        .stream_position()
        .map_err(super::super::local_store::io_error)?;
    let size = input
        .metadata()
        .map_err(super::super::local_store::io_error)?
        .len();
    if offset == size {
        return Ok(None);
    }
    let mut bytes = [0; HEADER];
    input
        .read_exact(&mut bytes)
        .map_err(|_| corrupt("truncated segment frame"))?;
    let position = u64::from_le_bytes(bytes[..8].try_into().unwrap());
    let commit = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let length = u64::from_le_bytes(bytes[16..].try_into().unwrap());
    if length == 0 || length > size.saturating_sub(offset + HEADER as u64) {
        return Err(corrupt("invalid native segment payload length"));
    }
    Ok(Some((position, commit, length)))
}
fn record_identity(
    positions: &mut std::collections::BTreeSet<u64>,
    commits: &mut std::collections::BTreeSet<u64>,
    position: u64,
    commit: u64,
) -> Result<(), DurabilityError> {
    if !positions.insert(position) || !commits.insert(commit) {
        return Err(corrupt("duplicate native segment frame identity"));
    }
    Ok(())
}
pub(crate) fn inventory(path: &Path) -> Result<Vec<CommitId>, DurabilityError> {
    let mut input = open(path)?;
    let mut commits = Vec::new();
    let mut positions = std::collections::BTreeSet::new();
    let mut identities = std::collections::BTreeSet::new();
    while let Some((position, commit, length)) = header(&mut input)? {
        record_identity(&mut positions, &mut identities, position, commit)?;
        input
            .seek(SeekFrom::Current(
                i64::try_from(length).map_err(|_| corrupt("segment length overflow"))?,
            ))
            .map_err(super::super::local_store::io_error)?;
        commits.push(CommitId(commit));
    }
    Ok(commits)
}
pub(crate) fn preflight(
    layout: &crate::durability::data::DurableStoreLayout,
) -> Result<(), DurabilityError> {
    let directory = layout.root_path.join("segments");
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(super::super::local_store::io_error)? {
        let path = entry.map_err(super::super::local_store::io_error)?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "worth-segment")
        {
            inventory(&path)?;
        }
    }
    Ok(())
}
pub(crate) fn append(
    path: &Path,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    bytes: &ExecutionImmutableBytes,
) -> Result<(), DurabilityError> {
    // One native store I/O gate covers selection, this append and manifest update.
    // No request is retained or invented during settlement/repair.
    let mut existing = if path.exists() {
        Some(open(path)?)
    } else {
        None
    };
    if let Some(input) = existing.as_mut() {
        if validate_existing(input, positioned, bytes)? {
            return Ok(());
        }
        input
            .rewind()
            .map_err(super::super::local_store::io_error)?;
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(super::super::local_store::io_error)?;
    }
    let temporary = path.with_extension("tmp");
    let mut output = fs::File::create(&temporary).map_err(super::super::local_store::io_error)?;
    if let Some(mut input) = existing {
        let mut scratch = [0; 8192];
        loop {
            let count = input
                .read(&mut scratch)
                .map_err(super::super::local_store::io_error)?;
            if count == 0 {
                break;
            }
            output
                .write_all(&scratch[..count])
                .map_err(super::super::local_store::io_error)?;
        }
    } else {
        output
            .write_all(MAGIC)
            .map_err(super::super::local_store::io_error)?;
    }
    for part in [
        positioned.position().0.to_le_bytes(),
        positioned.envelope().commit.commit_id.0.to_le_bytes(),
        (bytes.bytes().len() as u64).to_le_bytes(),
    ] {
        output
            .write_all(&part)
            .map_err(super::super::local_store::io_error)?;
    }
    output
        .write_all(bytes.bytes())
        .map_err(super::super::local_store::io_error)?;
    output
        .sync_all()
        .map_err(super::super::local_store::io_error)?;
    drop(output);
    fs::rename(&temporary, path).map_err(super::super::local_store::io_error)
}
fn validate_existing(
    input: &mut fs::File,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    bytes: &ExecutionImmutableBytes,
) -> Result<bool, DurabilityError> {
    let mut already_written = false;
    let mut positions = std::collections::BTreeSet::new();
    let mut identities = std::collections::BTreeSet::new();
    while let Some((position, commit, length)) = header(input)? {
        record_identity(&mut positions, &mut identities, position, commit)?;
        if position == positioned.position().0 || commit == positioned.envelope().commit.commit_id.0
        {
            if position != positioned.position().0
                || commit != positioned.envelope().commit.commit_id.0
                || length != bytes.bytes().len() as u64
            {
                return Err(corrupt("durable append identity mismatch"));
            }
            let mut offset = 0;
            let mut chunk = [0; 8192];
            while offset < bytes.bytes().len() {
                let count = chunk.len().min(bytes.bytes().len() - offset);
                input
                    .read_exact(&mut chunk[..count])
                    .map_err(super::super::local_store::io_error)?;
                if chunk[..count] != bytes.bytes()[offset..offset + count] {
                    return Err(corrupt("durable append payload mismatch"));
                }
                offset += count;
            }
            already_written = true;
            continue; // still validate every following frame before accepting repair
        }
        input
            .seek(SeekFrom::Current(
                i64::try_from(length).map_err(|_| corrupt("segment length overflow"))?,
            ))
            .map_err(super::super::local_store::io_error)?;
    }
    Ok(already_written)
}

pub(crate) fn read(
    execution: &ExecutionRequest<'_, '_>,
    path: &Path,
) -> Result<DurableSegmentFile, RelationalRecoveryStop> {
    // Refuse old formats before allocating a payload buffer.
    drop(open(path)?);
    let bytes = super::text_readmission::read_payload(execution, path)?;
    let mut offset = MAGIC.len();
    let mut entries = Vec::new();
    let mut positions = std::collections::BTreeSet::new();
    let mut identities = std::collections::BTreeSet::new();
    while offset < bytes.bytes().len() {
        let frame = bytes
            .bytes()
            .get(offset..offset + HEADER)
            .ok_or_else(|| corrupt("truncated segment frame"))?;
        let position = PatchStreamPosition(u64::from_le_bytes(frame[..8].try_into().unwrap()));
        let commit = CommitId(u64::from_le_bytes(frame[8..16].try_into().unwrap()));
        record_identity(&mut positions, &mut identities, position.0, commit.0)?;
        let length = usize::try_from(u64::from_le_bytes(frame[16..].try_into().unwrap()))
            .map_err(|_| corrupt("segment length overflow"))?;
        offset += HEADER;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| corrupt("segment length overflow"))?;
        let payload = bytes
            .bytes()
            .get(offset..end)
            .ok_or_else(|| corrupt("truncated native payload"))?;
        let wire: crate::history::data::CanonicalEnvelopeWire<'_> = rmp_serde::from_slice(payload)
            .map_err(|error| corrupt(&format!("current native envelope: {error}")))?;
        let canonical = wire
            .readmit(execution)
            .map_err(crate::symbols::data::field_text_recovery_stop)?;
        if canonical.commit.commit_id != commit {
            return Err(corrupt("segment header and envelope disagree").into());
        }
        entries.push(
            super::super::persisted_canonical_commit::PersistedCanonicalCommit::from_parts(
                position, canonical,
            ),
        );
        offset = end;
    }
    Ok(DurableSegmentFile { entries })
}
#[cfg(test)]
pub(crate) fn write(
    execution: &ExecutionRequest<'_, '_>,
    path: &Path,
    file: &DurableSegmentFile,
) -> Result<(), RelationalRecoveryStop> {
    let temporary = path.with_extension("fixture");
    if temporary.exists() {
        fs::remove_file(&temporary).map_err(super::super::local_store::io_error)?;
    }
    for entry in &file.entries {
        let positioned = entry
            .clone()
            .readmit()
            .map_err(|detail| corrupt(&detail))?
            .into_positioned();
        let bytes = super::encode_payload(positioned.envelope(), execution.allocation_policy())
            .map_err(|denial| match denial {
                crate::durability::data::RelationalNativeCheckpointCaptureDenial::Allocation(
                    error,
                ) => crate::symbols::data::field_text_recovery_stop(
                    crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(error),
                ),
                crate::durability::data::RelationalNativeCheckpointCaptureDenial::Durability(
                    error,
                ) => error.into(),
            })?;
        append(&temporary, &positioned, &bytes)?;
    }
    fs::rename(temporary, path).map_err(super::super::local_store::io_error)?;
    Ok(())
}
