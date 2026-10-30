pub use std::path::Path;

pub use crate::durability_admission::independent_wal_oracle::BindingInspectionDenial;

pub const RETIREMENT_DOMAIN: &[u8] = b"store.physical.retirement.v2";
pub const REWRITE_DOMAIN: &[u8] = b"store.physical.rewrite-redo.v2";
pub const FRAME_HEADER_BYTES: usize = 116;
pub const FRAME_FOOTER_BYTES: usize = 32;
pub const PAYLOAD_LENGTH_AT: usize = 44;
pub const BODY_BYTES: usize = 121;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndependentRetirementAction {
    Intent,
    Completion,
}

/// Which artifact family the retired generation belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndependentRetiredKind {
    Segment,
    Extent,
    Arena,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndependentRetirementRedo {
    pub action: IndependentRetirementAction,
    pub kind: IndependentRetiredKind,
    pub source_root: u64,
    pub artifact_id: u64,
    pub generation: u64,
    pub bytes: u64,
    pub arena_range: Option<[u64; 3]>,
    pub release_roots: Option<[u64; 2]>,
    pub release_digest: Option<[u8; 32]>,
    pub metadata_bytes: u64,
    pub publication: u64,
}

/// Decodes one whole WAL frame payload as a retirement record without
/// importing the runtime codec.
pub fn inspect_retirement_redo(
    payload: &[u8],
) -> Result<IndependentRetirementRedo, BindingInspectionDenial> {
    let length = payload.get(..8).ok_or(
        crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated,
    )?;
    let length = u64::from_le_bytes(length.try_into().expect("fixed u64"));
    if length != RETIREMENT_DOMAIN.len() as u64
        || payload.get(8..8 + RETIREMENT_DOMAIN.len()) != Some(RETIREMENT_DOMAIN)
    {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::DomainMismatch);
    }
    let body = &payload[8 + RETIREMENT_DOMAIN.len()..];
    if body.len() != BODY_BYTES {
        return Err(if body.len() < BODY_BYTES {
            crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated
        } else {
            crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::TrailingBytes
        });
    }
    let (action, kind) = match body[0] {
        1 => (
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent,
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Segment,
        ),
        2 => (
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion,
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Segment,
        ),
        3 => (
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent,
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Extent,
        ),
        4 => (
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion,
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Extent,
        ),
        5 => (
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent,
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Arena,
        ),
        6 => (
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion,
            crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Arena,
        ),
        _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
    };
    let number = |index: usize| {
        let start = 1 + index * 8;
        u64::from_le_bytes(body[start..start + 8].try_into().expect("fixed u64"))
    };
    let (arena_range, release_roots, release_digest) = match kind {
        crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Segment if body[33..] == [0; 88] => (None, None, None),
        crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Extent
            if number(4) != 0
                && number(6) == number(3)
                && number(6) != 0
                && number(5).checked_add(number(6)).is_some()
                && number(7) > number(0)
                && number(7).checked_add(1) == Some(number(8))
                && body[73..105] != [0; 32]
                && number(13) != 0
                && number(14) != 0 =>
        {
            (
                Some([number(4), number(5), number(6)]),
                Some([number(7), number(8)]),
                Some(body[73..105].try_into().unwrap()),
            )
        }
        crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Arena
            if body[33..57] == [0; 24]
                && number(2) == number(0)
                && number(7) >= number(0)
                && number(7).checked_add(1) == Some(number(8))
                && body[73..105] != [0; 32]
                && number(13) != 0
                && number(14) != 0 =>
        {
            (
                None,
                Some([number(7), number(8)]),
                Some(body[73..105].try_into().unwrap()),
            )
        }
        _ => return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame),
    };
    if [number(0), number(1), number(2)].contains(&0) {
        return Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame);
    }
    Ok(IndependentRetirementRedo {
        action,
        kind,
        source_root: number(0),
        artifact_id: number(1),
        generation: number(2),
        bytes: number(3),
        arena_range,
        release_roots,
        release_digest,
        metadata_bytes: number(13),
        publication: number(14),
    })
}

/// Every retirement record in the store's WAL, in file and frame order.
pub fn produced_retirement_payloads(store_root: &Path) -> Vec<IndependentRetirementRedo> {
    let mut files = Vec::new();
    collect_files(&store_root.join("families").join("wal"), &mut files);
    files.sort();
    files
        .iter()
        .flat_map(|path| file_retirement_payloads(path))
        .collect()
}

/// Retirement records framed in one WAL segment file. A payload that claims
/// the retirement domain but fails to decode is a codec defect, not absence.
pub fn file_retirement_payloads(path: &Path) -> Vec<IndependentRetirementRedo> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut offset = 0;
    while let Some(payload) = frame_payload(&bytes, offset) {
        offset += FRAME_HEADER_BYTES + payload.len() + FRAME_FOOTER_BYTES;
        match inspect_retirement_redo(payload) {
            Ok(record) => found.push(record),
            Err(crate::durability_admission::independent_wal_oracle::BindingInspectionDenial::DomainMismatch) => {}
            Err(denial) => panic!("retirement frame in {path:?} is malformed: {denial:?}"),
        }
    }
    found
}

pub fn frame_payload(bytes: &[u8], offset: usize) -> Option<&[u8]> {
    let header = bytes.get(offset..offset.checked_add(FRAME_HEADER_BYTES)?)?;
    if header.get(..8) != Some(b"WORTHWAL".as_slice()) {
        return None;
    }
    let length = header.get(PAYLOAD_LENGTH_AT..PAYLOAD_LENGTH_AT + 8)?;
    let length = usize::try_from(u64::from_le_bytes(length.try_into().ok()?)).ok()?;
    let start = offset + FRAME_HEADER_BYTES;
    let end = start.checked_add(length)?;
    bytes.get(end..end.checked_add(FRAME_FOOTER_BYTES)?)?;
    bytes.get(start..end)
}

pub fn collect_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}
