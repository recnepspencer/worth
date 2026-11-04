//! Closed version-one directory; all ranges remain in the actual owned input.
use super::{
    authentication, refusal::CANONICAL_MEANINGS,
    CheckpointAdmissionAuthenticationMismatch as Authentication,
    CheckpointAdmissionMeaning as Meaning, RelationalCheckpointAdmissionRefusal as Refusal,
};
mod emission;
pub(super) use emission::encode;
use sha2::{Digest, Sha256};
use std::ops::Range;
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionOrderedMap, ExecutionRequest,
    ExecutionRequestArrayBuilder,
};
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion, CheckpointAdmissionCut,
};

pub(super) const MAGIC: &[u8; 8] = b"WRTHCAI1";
pub(super) const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
pub(super) const HEADER: usize = 64;
const GROUP_HEADER: usize = 41;
const MINIMUM_ROW: usize = 16 + 8 * GROUP_HEADER;
// The cut contains only closed field names, fixed arrays and bounded integers.
const MAXIMUM_CUT: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AdmissionRowKey {
    pub(crate) position: u64,
    pub(crate) commit: u64,
}
#[derive(Debug, Clone)]
pub(crate) struct AdmissionRowIndex {
    pub(crate) key: AdmissionRowKey,
    pub(crate) groups: [Range<usize>; 8],
}
pub(crate) struct EncodedAdmissionRow {
    pub(crate) key: AdmissionRowKey,
    pub(crate) groups: [ExecutionImmutableBytes; 8],
}
pub(super) struct DecodedDirectory {
    pub(super) cut: CheckpointAdmissionCut,
    pub(super) roots: Range<usize>,
    pub(super) rows: ExecutionArray<AdmissionRowIndex>,
    pub(super) commits: ExecutionOrderedMap<u64, usize>,
    pub(super) positions: ExecutionOrderedMap<
        crate::history::data::CommitId,
        crate::history::data::RequiredOutputPublicationPositions,
    >,
}

pub(super) fn read(
    bytes: &ExecutionImmutableBytes,
    region: Range<usize>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<DecodedDirectory, Refusal> {
    request
        .consult()
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let raw = bytes
        .get(region.clone())
        .filter(|raw| raw.len() <= 1 << 30)
        .ok_or(Refusal::Malformed)?;
    if raw.get(..8) != Some(MAGIC.as_slice()) {
        return Err(Refusal::Malformed);
    }
    let produced = u32::from_le_bytes(
        raw.get(8..12)
            .ok_or(Refusal::Malformed)?
            .try_into()
            .map_err(|_| Refusal::Malformed)?,
    );
    let produced =
        BoundaryProtocolVersion::try_new(produced).map_err(|_| Refusal::InvalidVersion)?;
    BoundaryProtocolCompatibilityWindow::inclusive(VERSION, VERSION)
        .admit(produced)
        .map_err(Refusal::UnsupportedVersion)?;
    let cut_length = u32::from_le_bytes(
        raw.get(12..16)
            .ok_or(Refusal::Malformed)?
            .try_into()
            .map_err(|_| Refusal::Malformed)?,
    ) as usize;
    let row_count = usize::try_from(integer(raw, 16)?).map_err(|_| Refusal::CapacityOverflow)?;
    let root_length = usize::try_from(integer(raw, 24)?).map_err(|_| Refusal::CapacityOverflow)?;
    if cut_length > MAXIMUM_CUT || row_count > raw.len() / MINIMUM_ROW {
        return Err(Refusal::Malformed);
    }
    let cut_end = HEADER
        .checked_add(cut_length)
        .ok_or(Refusal::CapacityOverflow)?;
    let cut_bytes = raw.get(HEADER..cut_end).ok_or(Refusal::Malformed)?;
    let cut: CheckpointAdmissionCut =
        rmp_serde::from_slice(cut_bytes).map_err(|_| Refusal::Malformed)?;
    if row_count as u64 != cut.canonical_through() {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let canonical_cut =
        crate::durability::artifact_payload_emission::request::encode_named_charged(
            &[],
            &super::binary_encoding::BinaryFixedArrays(&cut),
            request,
        )
        .map_err(super::refusal::native_stop)?;
    if canonical_cut.bytes() != cut_bytes {
        return Err(Refusal::Malformed);
    }
    let mut directory =
        authentication::directory(raw.get(..32).ok_or(Refusal::Malformed)?, cut_bytes, request)?;
    let mut offset = cut_end;
    let roots = group(
        raw,
        &mut offset,
        cut_bytes,
        AdmissionRowKey {
            position: 0,
            commit: 0,
        },
        Meaning::RootDirectory,
        &mut directory,
        request,
    )?;
    if roots.len() != root_length {
        return Err(Refusal::Malformed);
    }
    let mut rows =
        ExecutionRequestArrayBuilder::allocate(*request, row_count).map_err(Refusal::Buffer)?;
    let mut previous_position = 0;
    let mut commits = ExecutionOrderedMap::default();
    for row_number in 0..row_count {
        let position = integer(raw, offset)?;
        let commit = integer(raw, offset.checked_add(8).ok_or(Refusal::CapacityOverflow)?)?;
        offset = offset.checked_add(16).ok_or(Refusal::CapacityOverflow)?;
        if position == 0
            || commit == 0
            || position <= previous_position
            || position > cut.canonical_through()
            || commits.contains_key(&commit)
        {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        let key = AdmissionRowKey { position, commit };
        authentication::update(&mut directory, &position.to_le_bytes(), request)?;
        authentication::update(&mut directory, &commit.to_le_bytes(), request)?;
        let mut groups = std::array::from_fn(|_| 0..0);
        for (index, meaning) in CANONICAL_MEANINGS.into_iter().enumerate() {
            let range = group(
                raw,
                &mut offset,
                cut_bytes,
                key,
                meaning,
                &mut directory,
                request,
            )?;
            groups[index] = region.start + range.start..region.start + range.end;
        }
        rows.push(AdmissionRowIndex { key, groups })
            .map_err(Refusal::Buffer)?;
        commits
            .insert_cloned_native_for_request(&commit, &row_number, *request)
            .map_err(|cause| {
                super::refusal::native_stop(
                    crate::durability::checkpoint_history::capture::map_stop(cause),
                )
            })?;
        previous_position = position;
    }
    if offset != raw.len() {
        return Err(Refusal::Malformed);
    }
    if raw.get(32..HEADER) != Some(directory.finalize().as_slice()) {
        return Err(Refusal::AuthenticationMismatch(Authentication::Directory));
    }
    let positions = super::root_positions::read(&raw[roots.clone()], &commits, request)?;
    Ok(DecodedDirectory {
        cut,
        roots: region.start + roots.start..region.start + roots.end,
        rows: rows.seal().map_err(Refusal::Buffer)?,
        commits,
        positions,
    })
}

fn group(
    raw: &[u8],
    offset: &mut usize,
    cut: &[u8],
    key: AdmissionRowKey,
    expected: Meaning,
    directory: &mut Sha256,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Range<usize>, Refusal> {
    let start = *offset;
    let header_end = start
        .checked_add(GROUP_HEADER)
        .ok_or(Refusal::CapacityOverflow)?;
    let header = raw.get(start..header_end).ok_or(Refusal::Malformed)?;
    if header[0] != expected as u8 {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let length = usize::try_from(integer(header, 1)?).map_err(|_| Refusal::CapacityOverflow)?;
    let end = header_end
        .checked_add(length)
        .ok_or(Refusal::CapacityOverflow)?;
    let payload = raw.get(header_end..end).ok_or(Refusal::Malformed)?;
    let digest = authentication::meaning(cut, key, expected, payload, request)?;
    if header[9..] != digest {
        return Err(Refusal::AuthenticationMismatch(Authentication::Meaning {
            position: key.position,
            meaning: expected,
        }));
    }
    authentication::update(directory, header, request)?;
    *offset = end;
    Ok(header_end..end)
}

fn integer(raw: &[u8], offset: usize) -> Result<u64, Refusal> {
    let end = offset.checked_add(8).ok_or(Refusal::CapacityOverflow)?;
    Ok(u64::from_le_bytes(
        raw.get(offset..end)
            .ok_or(Refusal::Malformed)?
            .try_into()
            .map_err(|_| Refusal::Malformed)?,
    ))
}
