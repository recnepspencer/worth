//! Mutate one real canonical frame while preserving its indexed and outer digests.
use super::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::ops::Range;
fn length(bytes: &[u8]) -> usize {
    let mut decoder = rmp_serde::Deserializer::new(std::io::Cursor::new(bytes));
    serde::de::IgnoredAny::deserialize(&mut decoder).unwrap();
    decoder.position() as usize
}
fn count(bytes: &[u8], map: bool) -> (usize, usize) {
    match bytes[0] {
        marker if (if map { 0x80..=0x8f } else { 0x90..=0x9f }).contains(&marker) => {
            ((marker & 15) as usize, 1)
        }
        marker if marker == if map { 0xde } else { 0xdc } => (
            u16::from_be_bytes(bytes[1..3].try_into().unwrap()) as usize,
            3,
        ),
        marker if marker == if map { 0xdf } else { 0xdd } => (
            u32::from_be_bytes(bytes[1..5].try_into().unwrap()) as usize,
            5,
        ),
        marker => panic!("unexpected collection marker {marker:x}"),
    }
}
pub(super) fn optional_field(bytes: &[u8], name: &str) -> Option<Range<usize>> {
    let (count, mut cursor) = count(bytes, true);
    for _ in 0..count {
        let end = cursor + length(&bytes[cursor..]);
        let key: String = rmp_serde::from_slice(&bytes[cursor..end]).unwrap();
        cursor = end;
        let value = cursor..cursor + length(&bytes[cursor..]);
        if key == name {
            return Some(value);
        }
        cursor = value.end;
    }
    None
}
pub(super) fn field(bytes: &[u8], name: &str) -> Range<usize> {
    optional_field(bytes, name).unwrap_or_else(|| panic!("missing checkpoint field {name}"))
}
pub(super) fn items(bytes: &[u8]) -> Vec<Range<usize>> {
    let (count, mut cursor) = count(bytes, false);
    (0..count)
        .map(|_| {
            let value = cursor..cursor + length(&bytes[cursor..]);
            cursor = value.end;
            value
        })
        .collect()
}
fn binary(bytes: &[u8]) -> Range<usize> {
    let (count, header) = match bytes[0] {
        0xc4 => (bytes[1] as usize, 2),
        0xc5 => (
            u16::from_be_bytes(bytes[1..3].try_into().unwrap()) as usize,
            3,
        ),
        0xc6 => (
            u32::from_be_bytes(bytes[1..5].try_into().unwrap()) as usize,
            5,
        ),
        marker => panic!("expected envelope bytes, got {marker:x}"),
    };
    header..header + count
}
fn digest(blob: &[u8]) -> [u8; 32] {
    let native = field(blob, "Native");
    let fields = &blob[native];
    let position = field(fields, "position");
    let canonical = field(fields, "canonical");
    let mut canonical = fields[canonical].to_vec();
    let diagnostics = field(&canonical, "diagnostics_summary");
    let entries = field(&canonical[diagnostics.clone()], "entries");
    canonical.splice(
        diagnostics.start + entries.start..diagnostics.start + entries.end,
        [0x90],
    );
    // Reconstruct the authoritative projection, which omits diagnostic entries.
    // Its digest is checked against the original index before any corruption.
    let mut payload = vec![0x82];
    payload.extend(rmp_serde::to_vec("position").unwrap());
    payload.extend(&fields[position]);
    payload.extend(rmp_serde::to_vec("canonical").unwrap());
    payload.extend(&canonical);
    Sha256::digest(payload).into()
}
pub(super) fn mutate(
    checkpoint: &WorthQueryApplicationCheckpoint,
    mut change: impl FnMut(&[u8]) -> Option<(Range<usize>, Vec<u8>)>,
) -> WorthQueryApplicationCheckpoint {
    let original = checkpoint.bytes();
    let magic = b"WORTH-REL-CHK20\n";
    let start = original
        .windows(magic.len())
        .position(|value| value == magic)
        .unwrap();
    let native_len =
        u64::from_be_bytes(original[start - 16..start - 8].try_into().unwrap()) as usize;
    let body_start = start + magic.len();
    let wire = &original[body_start..start + native_len];
    let envelopes = field(wire, "envelopes");
    for item in items(&wire[envelopes.clone()]) {
        let bin = envelopes.start + item.start;
        let payload = binary(&wire[bin..]);
        let blob_start = bin + payload.start;
        let blob = &wire[blob_start..bin + payload.end];
        let native = field(blob, "Native");
        let canonical = field(&blob[native.clone()], "canonical");
        let envelope_start = native.start + canonical.start;
        let envelope = &blob[envelope_start..native.start + canonical.end];
        let Some((range, replacement)) = change(envelope) else {
            continue;
        };
        let offset = envelope_start + range.start;
        let mut changed = blob.to_vec();
        changed.splice(offset..envelope_start + range.end, replacement);
        let position = field(&blob[native.clone()], "position");
        let position = &blob[native][position];
        let manifest = field(wire, "manifest");
        let commits = field(&wire[manifest.clone()], "commits");
        for commit in items(&wire[manifest.clone()][commits.clone()]) {
            let entry_start = manifest.start + commits.start + commit.start;
            let entry = &wire[entry_start..manifest.start + commits.start + commit.end];
            if &entry[field(entry, "position")] != position {
                continue;
            }
            let indexed = field(entry, "payload_digest");
            assert_eq!(
                &entry[indexed.clone()],
                rmp_serde::to_vec(&digest(blob)).unwrap(),
                "original artifact digest reconstruction is byte-exact"
            );
            let indexed_replacement = rmp_serde::to_vec(&digest(&changed)).unwrap();
            let mut encoded_blob = Vec::new();
            if changed.len() <= u8::MAX as usize {
                encoded_blob.extend([0xc4, changed.len() as u8]);
            } else if changed.len() <= u16::MAX as usize {
                encoded_blob.push(0xc5);
                encoded_blob.extend((changed.len() as u16).to_be_bytes());
            } else {
                encoded_blob.push(0xc6);
                encoded_blob.extend((changed.len() as u32).to_be_bytes());
            }
            encoded_blob.extend(changed);
            let mut edits = vec![
                (
                    body_start + bin..body_start + bin + payload.end,
                    encoded_blob,
                ),
                (
                    body_start + entry_start + indexed.start
                        ..body_start + entry_start + indexed.end,
                    indexed_replacement,
                ),
            ];
            let delta: isize = edits
                .iter()
                .map(|(range, bytes)| bytes.len() as isize - range.len() as isize)
                .sum();
            edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
            let mut frame = original.to_vec();
            for (range, bytes) in edits {
                frame.splice(range, bytes);
            }
            frame[start - 16..start - 8]
                .copy_from_slice(&((native_len as isize + delta) as u64).to_be_bytes());
            let checksum = Sha256::digest(&frame[40..]);
            frame[8..40].copy_from_slice(&checksum);
            return WorthQueryApplicationCheckpoint::from_untrusted_bytes(frame.into_boxed_slice());
        }
        panic!("selected frame has no checkpoint index entry");
    }
    panic!("selected authentic history frame is absent")
}
