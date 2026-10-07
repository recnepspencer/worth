use std::path::Path;

use super::BindingInspectionDenial;

const RETIREMENT_DOMAIN: &[u8] = b"store.physical.retirement.v2";
const REWRITE_DOMAIN: &[u8] = b"store.physical.rewrite-redo.v2";
const FRAME_HEADER_BYTES: usize = 116;
const FRAME_FOOTER_BYTES: usize = 32;
const PAYLOAD_LENGTH_AT: usize = 44;
const BODY_BYTES: usize = 121;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IndependentRetirementAction {
    Intent,
    Completion,
}

/// Which artifact family the retired generation belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IndependentRetiredKind {
    Segment,
    Extent,
    Arena,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IndependentRetirementRedo {
    pub(crate) action: IndependentRetirementAction,
    pub(crate) kind: IndependentRetiredKind,
    pub(crate) source_root: u64,
    pub(crate) artifact_id: u64,
    pub(crate) generation: u64,
    pub(crate) bytes: u64,
    pub(crate) arena_range: Option<[u64; 3]>,
    pub(crate) release_roots: Option<[u64; 2]>,
    pub(crate) release_digest: Option<[u8; 32]>,
    pub(crate) metadata_bytes: u64,
    pub(crate) publication: u64,
}

/// Decodes one whole WAL frame payload as a retirement record without
/// importing the runtime codec.
pub(in super::super) fn inspect_retirement_redo(
    payload: &[u8],
) -> Result<IndependentRetirementRedo, BindingInspectionDenial> {
    let length = payload.get(..8).ok_or(BindingInspectionDenial::Truncated)?;
    let length = u64::from_le_bytes(length.try_into().expect("fixed u64"));
    if length != RETIREMENT_DOMAIN.len() as u64
        || payload.get(8..8 + RETIREMENT_DOMAIN.len()) != Some(RETIREMENT_DOMAIN)
    {
        return Err(BindingInspectionDenial::DomainMismatch);
    }
    let body = &payload[8 + RETIREMENT_DOMAIN.len()..];
    if body.len() != BODY_BYTES {
        return Err(if body.len() < BODY_BYTES {
            BindingInspectionDenial::Truncated
        } else {
            BindingInspectionDenial::TrailingBytes
        });
    }
    let (action, kind) = match body[0] {
        1 => (
            IndependentRetirementAction::Intent,
            IndependentRetiredKind::Segment,
        ),
        2 => (
            IndependentRetirementAction::Completion,
            IndependentRetiredKind::Segment,
        ),
        3 => (
            IndependentRetirementAction::Intent,
            IndependentRetiredKind::Extent,
        ),
        4 => (
            IndependentRetirementAction::Completion,
            IndependentRetiredKind::Extent,
        ),
        5 => (
            IndependentRetirementAction::Intent,
            IndependentRetiredKind::Arena,
        ),
        6 => (
            IndependentRetirementAction::Completion,
            IndependentRetiredKind::Arena,
        ),
        _ => return Err(BindingInspectionDenial::InvalidFrame),
    };
    let number = |index: usize| {
        let start = 1 + index * 8;
        u64::from_le_bytes(body[start..start + 8].try_into().expect("fixed u64"))
    };
    let (arena_range, release_roots, release_digest) = match kind {
        IndependentRetiredKind::Segment if body[33..] == [0; 88] => (None, None, None),
        IndependentRetiredKind::Extent
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
        IndependentRetiredKind::Arena
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
        _ => return Err(BindingInspectionDenial::InvalidFrame),
    };
    if [number(0), number(1), number(2)].contains(&0) {
        return Err(BindingInspectionDenial::InvalidFrame);
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
pub(crate) fn produced_retirement_payloads(store_root: &Path) -> Vec<IndependentRetirementRedo> {
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
pub(crate) fn file_retirement_payloads(path: &Path) -> Vec<IndependentRetirementRedo> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut offset = 0;
    while let Some(payload) = frame_payload(&bytes, offset) {
        offset += FRAME_HEADER_BYTES + payload.len() + FRAME_FOOTER_BYTES;
        match inspect_retirement_redo(payload) {
            Ok(record) => found.push(record),
            Err(BindingInspectionDenial::DomainMismatch) => {}
            Err(denial) => panic!("retirement frame in {path:?} is malformed: {denial:?}"),
        }
    }
    found
}

fn frame_payload(bytes: &[u8], offset: usize) -> Option<&[u8]> {
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

fn collect_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Intent for source root 4, segment 1, generation 2, 16 bytes, written
    /// out field by field rather than through any encoder.
    fn golden_intent() -> Vec<u8> {
        let mut bytes = vec![28, 0, 0, 0, 0, 0, 0, 0];
        bytes.extend_from_slice(b"store.physical.retirement.v2");
        bytes.push(1);
        bytes.extend_from_slice(&[4, 0, 0, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&[2, 0, 0, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&[16, 0, 0, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&[0; 88]);
        bytes
    }

    #[test]
    fn retirement_golden_vectors_reject_unknown_and_malformed_payloads() {
        assert_eq!(golden_intent().len(), 8 + 28 + BODY_BYTES);
        assert_eq!(
            inspect_retirement_redo(&golden_intent()),
            Ok(IndependentRetirementRedo {
                action: IndependentRetirementAction::Intent,
                kind: IndependentRetiredKind::Segment,
                source_root: 4,
                artifact_id: 1,
                generation: 2,
                bytes: 16,
                arena_range: None,
                release_roots: None,
                release_digest: None,
                metadata_bytes: 0,
                publication: 0,
            })
        );
        let mut completion = golden_intent();
        completion[36] = 2;
        assert_eq!(
            inspect_retirement_redo(&completion).unwrap().action,
            IndependentRetirementAction::Completion
        );

        let mut truncated = golden_intent();
        truncated.pop();
        assert_eq!(
            inspect_retirement_redo(&truncated),
            Err(BindingInspectionDenial::Truncated)
        );
        let mut trailing = golden_intent();
        trailing.push(0);
        assert_eq!(
            inspect_retirement_redo(&trailing),
            Err(BindingInspectionDenial::TrailingBytes)
        );
        let mut extent_intent = golden_intent();
        extent_intent[36] = 3;
        extent_intent[69..77].copy_from_slice(&3u64.to_le_bytes());
        extent_intent[77..85].copy_from_slice(&4096u64.to_le_bytes());
        extent_intent[85..93].copy_from_slice(&16u64.to_le_bytes());
        extent_intent[93..101].copy_from_slice(&8u64.to_le_bytes());
        extent_intent[101..109].copy_from_slice(&9u64.to_le_bytes());
        extent_intent[109..141].fill(0x42);
        extent_intent[141..149].copy_from_slice(&4096u64.to_le_bytes());
        extent_intent[149..157].copy_from_slice(&71u64.to_le_bytes());
        let decoded = inspect_retirement_redo(&extent_intent).unwrap();
        assert_eq!(
            (decoded.action, decoded.kind),
            (
                IndependentRetirementAction::Intent,
                IndependentRetiredKind::Extent
            )
        );
        let mut extent_completion = extent_intent.clone();
        extent_completion[36] = 4;
        let decoded = inspect_retirement_redo(&extent_completion).unwrap();
        assert_eq!(
            (decoded.action, decoded.kind),
            (
                IndependentRetirementAction::Completion,
                IndependentRetiredKind::Extent
            )
        );
        let mut unknown_action = golden_intent();
        unknown_action[36] = 7;
        assert_eq!(
            inspect_retirement_redo(&unknown_action),
            Err(BindingInspectionDenial::InvalidFrame)
        );
        let mut rewrite = (REWRITE_DOMAIN.len() as u64).to_le_bytes().to_vec();
        rewrite.extend_from_slice(REWRITE_DOMAIN);
        assert_eq!(
            inspect_retirement_redo(&rewrite),
            Err(BindingInspectionDenial::DomainMismatch)
        );
    }
}
