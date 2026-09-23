use std::path::Path;

use super::BindingInspectionDenial;

const RETIREMENT_DOMAIN: &[u8] = b"store.physical.retirement.v1";
const REWRITE_DOMAIN: &[u8] = b"store.physical.rewrite-redo.v1";
const FRAME_HEADER_BYTES: usize = 116;
const FRAME_FOOTER_BYTES: usize = 32;
const PAYLOAD_LENGTH_AT: usize = 44;
const BODY_BYTES: usize = 1 + 4 * 8;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IndependentRetirementRedo {
    pub(crate) action: IndependentRetirementAction,
    pub(crate) kind: IndependentRetiredKind,
    pub(crate) source_root: u64,
    pub(crate) artifact_id: u64,
    pub(crate) generation: u64,
    pub(crate) bytes: u64,
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
        _ => return Err(BindingInspectionDenial::InvalidFrame),
    };
    let number = |index: usize| {
        let start = 1 + index * 8;
        u64::from_le_bytes(body[start..start + 8].try_into().expect("fixed u64"))
    };
    Ok(IndependentRetirementRedo {
        action,
        kind,
        source_root: number(0),
        artifact_id: number(1),
        generation: number(2),
        bytes: number(3),
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
        bytes.extend_from_slice(b"store.physical.retirement.v1");
        bytes.push(1);
        bytes.extend_from_slice(&[4, 0, 0, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&[2, 0, 0, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&[16, 0, 0, 0, 0, 0, 0, 0]);
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
        let decoded = inspect_retirement_redo(&extent_intent).unwrap();
        assert_eq!(
            (decoded.action, decoded.kind),
            (
                IndependentRetirementAction::Intent,
                IndependentRetiredKind::Extent
            )
        );
        let mut extent_completion = golden_intent();
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
        unknown_action[36] = 5;
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
