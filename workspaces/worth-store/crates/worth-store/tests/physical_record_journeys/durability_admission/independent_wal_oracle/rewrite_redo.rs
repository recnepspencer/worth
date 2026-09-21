use std::fs;
use std::path::Path;

use super::{BindingField, BindingInspectionDenial};

const REWRITE_DOMAIN: &[u8] = b"store.physical.rewrite-redo.v1";
const CANONICAL_DOMAIN: &[u8] = b"store.physical.wal.canonical-redo.v3";
const BODY_BYTES: usize = 216;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) struct IndependentRewriteRedo {
    pub(in super::super) source_root_generation: u64,
    pub(in super::super) source_length: u32,
    pub(in super::super) destination_length: u32,
    pub(in super::super) candidate_bytes: u64,
    pub(in super::super) resulting_root_generation: u64,
}

pub(in super::super) fn inspect_rewrite_redo(
    bytes: &[u8],
) -> Result<IndependentRewriteRedo, BindingInspectionDenial> {
    let mut cursor = bytes;
    let domain = take_field(&mut cursor)?;
    if domain == CANONICAL_DOMAIN {
        return Err(BindingInspectionDenial::DomainMismatch);
    }
    if domain != REWRITE_DOMAIN {
        return Err(BindingInspectionDenial::DomainMismatch);
    }
    if cursor.len() != BODY_BYTES {
        return Err(if cursor.len() < BODY_BYTES {
            BindingInspectionDenial::Truncated
        } else {
            BindingInspectionDenial::TrailingBytes
        });
    }
    cursor = &cursor[32 + 32..];
    let source_root_generation = take_u64(&mut cursor)?;
    let _source_generation = take_u64(&mut cursor)?;
    let _source_offset = take_u64(&mut cursor)?;
    let source_length = take_u32(&mut cursor)?;
    cursor = &cursor[32..];
    let _destination_generation = take_u64(&mut cursor)?;
    let _destination_offset = take_u64(&mut cursor)?;
    let destination_length = take_u32(&mut cursor)?;
    let _page_lsn = take_u64(&mut cursor)?;
    cursor = &cursor[32..];
    let _source_placement = take_u64(&mut cursor)?;
    let _destination_placement = take_u64(&mut cursor)?;
    let candidate_bytes = take_u64(&mut cursor)?;
    let resulting_root_generation = take_u64(&mut cursor)?;
    if !cursor.is_empty() {
        return Err(BindingInspectionDenial::TrailingBytes);
    }
    if source_length == 0
        || source_length != destination_length
        || u64::from(destination_length) != candidate_bytes
        || source_root_generation == 0
        || resulting_root_generation != source_root_generation.saturating_add(1)
    {
        return Err(BindingInspectionDenial::InvalidFrame);
    }
    Ok(IndependentRewriteRedo {
        source_root_generation,
        source_length,
        destination_length,
        candidate_bytes,
        resulting_root_generation,
    })
}

pub(in super::super) fn produced_rewrite_payloads(root: &Path) -> Vec<IndependentRewriteRedo> {
    let mut found = Vec::new();
    collect(root, &mut found);
    found
}

fn collect(root: &Path, found: &mut Vec<IndependentRewriteRedo>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let mut offset = 0;
        while let Some(index) = bytes[offset..]
            .windows(REWRITE_DOMAIN.len())
            .position(|window| window == REWRITE_DOMAIN)
        {
            let domain_at = offset + index;
            if let Some(payload) = framed_payload(&bytes, domain_at) {
                if let Ok(rewrite) = inspect_rewrite_redo(payload) {
                    found.push(rewrite);
                }
            }
            offset = domain_at + REWRITE_DOMAIN.len();
        }
    }
}

fn framed_payload(bytes: &[u8], domain_at: usize) -> Option<&[u8]> {
    let length_at = domain_at.checked_sub(8)?;
    let domain_len = u64::from_le_bytes(bytes.get(length_at..domain_at)?.try_into().ok()?);
    if domain_len != REWRITE_DOMAIN.len() as u64 {
        return None;
    }
    let redo_len_at = length_at.checked_sub(8)?;
    let redo_len = u64::from_le_bytes(bytes.get(redo_len_at..length_at)?.try_into().ok()?);
    let payload_len = 8 + REWRITE_DOMAIN.len() + BODY_BYTES;
    if redo_len != payload_len as u64 {
        return None;
    }
    bytes.get(length_at..length_at + payload_len)
}

fn take_field<'a>(cursor: &mut &'a [u8]) -> Result<&'a [u8], BindingInspectionDenial> {
    let length = usize::try_from(take_u64(cursor)?)
        .map_err(|_| BindingInspectionDenial::InvalidFieldLength(BindingField::RedoPayload))?;
    take(cursor, length)
}

fn take_u64(cursor: &mut &[u8]) -> Result<u64, BindingInspectionDenial> {
    let bytes = take(cursor, 8)?;
    Ok(u64::from_le_bytes(bytes.try_into().expect("fixed u64")))
}

fn take_u32(cursor: &mut &[u8]) -> Result<u32, BindingInspectionDenial> {
    let bytes = take(cursor, 4)?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("fixed u32")))
}

fn take<'a>(cursor: &mut &'a [u8], length: usize) -> Result<&'a [u8], BindingInspectionDenial> {
    if cursor.len() < length {
        return Err(BindingInspectionDenial::Truncated);
    }
    let (value, rest) = cursor.split_at(length);
    *cursor = rest;
    Ok(value)
}

fn sample_rewrite() -> Vec<u8> {
    let mut body = vec![0_u8; BODY_BYTES];
    body[64..72].copy_from_slice(&4_u64.to_le_bytes());
    body[88..92].copy_from_slice(&4096_u32.to_le_bytes());
    body[140..144].copy_from_slice(&4096_u32.to_le_bytes());
    body[200..208].copy_from_slice(&4096_u64.to_le_bytes());
    body[208..216].copy_from_slice(&5_u64.to_le_bytes());
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&(REWRITE_DOMAIN.len() as u64).to_le_bytes());
    encoded.extend_from_slice(REWRITE_DOMAIN);
    encoded.extend_from_slice(&body);
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_golden_vectors_reject_unknown_and_malformed_payloads() {
        let rewrite = inspect_rewrite_redo(&sample_rewrite()).unwrap();
        assert_eq!(rewrite.candidate_bytes, 4096);
        assert_eq!(rewrite.source_length, rewrite.destination_length);

        let mut truncated = sample_rewrite();
        truncated.pop();
        assert_eq!(
            inspect_rewrite_redo(&truncated),
            Err(BindingInspectionDenial::Truncated)
        );

        let mut unknown = (b"store.physical.unknown.v9".len() as u64).to_le_bytes().to_vec();
        unknown.extend_from_slice(b"store.physical.unknown.v9");
        unknown.extend_from_slice(&[0; BODY_BYTES]);
        assert_eq!(
            inspect_rewrite_redo(&unknown),
            Err(BindingInspectionDenial::DomainMismatch)
        );

        let mut canonical = (CANONICAL_DOMAIN.len() as u64).to_le_bytes().to_vec();
        canonical.extend_from_slice(CANONICAL_DOMAIN);
        assert_eq!(
            inspect_rewrite_redo(&canonical),
            Err(BindingInspectionDenial::DomainMismatch)
        );

        let mut mismatch = sample_rewrite();
        let dest_len_at = 8 + REWRITE_DOMAIN.len() + 140;
        mismatch[dest_len_at..dest_len_at + 4].copy_from_slice(&4095_u32.to_le_bytes());
        assert_eq!(
            inspect_rewrite_redo(&mismatch),
            Err(BindingInspectionDenial::InvalidFrame)
        );
    }
}
