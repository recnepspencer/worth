//! Independent editor-result oracle. Coordinates and kinds come from the pristine
//! stream, never the changed framing or either integrity implementation.
use super::{ArtifactGranule, Op};
use sha2::{Digest, Sha256};

pub(super) fn audit(before: &[u8], after: &[u8], target: &ArtifactGranule, op: Op) {
    let records = baseline_records(before);
    let selected = records.iter().position(|r| r.0 == target.offset()).unwrap();
    let (start, old_length, kind) = records[selected];
    assert_eq!(old_length, target.length());
    let new_length = if kind == 4 && op == Op::ScopeSubstitution { 36 } else { old_length };
    assert_eq!(after.len(), before.len() - old_length + new_length);
    let old = &before[start..start + old_length];
    let changed = &after[start..start + new_length];
    target_bytes(old, changed, before, &records, op, kind);

    let mut dirty = Sha256::new();
    let mut bindings = Sha256::new();
    let mut dirty_count = 0;
    let mut binding_count = 0;
    let mut binding_bytes = 0;
    let mut compaction = None;
    for (index, &(offset, length, record_kind)) in records.iter().enumerate() {
        let shifted = if index > selected { offset - old_length + new_length } else { offset };
        let width = if index == selected { new_length } else { length };
        let frame = &after[shifted..shifted + width];
        if index != selected && record_kind != 5 {
            assert_eq!(frame, &before[offset..offset + length], "unrelated record changed");
        }
        match record_kind {
            2 => { dirty.update(frame); dirty_count += 1; }
            3 => { compaction = Some((shifted as u64, read64(&before[offset..], 16), read64(&before[offset..], 24))); }
            4 => { bindings.update(frame); binding_count += 1; binding_bytes += width as u64; }
            _ => {}
        }
    }
    let footer = &after[after.len() - 156..];
    let pristine_footer = &before[before.len() - 156..];
    if kind != 5 {
        if matches!(op, Op::CoveredByte | Op::SelectiveAggregate) {
            assert_eq!(footer, pristine_footer, "this operator must not refresh its enclosing footer");
        } else {
            for index in 0..156 {
                if !(48..88).contains(&index) && !(112..156).contains(&index) {
                    assert_eq!(footer[index], pristine_footer[index], "unrelated footer byte {index}");
                }
            }
        }
        assert!(valid_crc(footer));
    }
    assert_eq!(read64(footer, 40), dirty_count);
    assert_eq!(read64(footer, 104), binding_count);
    assert_eq!(read64(footer, 112), binding_bytes);
    let (offset, generation, cutoff) = compaction.unwrap();
    assert_eq!((read64(footer, 80), read64(footer, 88), read64(footer, 96)), (offset, generation, cutoff));
    let false_dirty = (kind == 2 && op == Op::CoveredByte)
        || (matches!(kind, 2 | 5) && op == Op::SelectiveAggregate);
    let false_binding = kind == 4 && matches!(op, Op::CoveredByte | Op::SelectiveAggregate);
    assert_eq!(footer[48..80] == dirty.finalize()[..], !false_dirty, "dirty selective summary isolation");
    assert_eq!(footer[120..152] == bindings.finalize()[..], !false_binding, "binding selective summary isolation");
}

fn target_bytes(old: &[u8], new: &[u8], baseline: &[u8], records: &[(usize, usize, u8)], op: Op, kind: u8) {
    assert_eq!(valid_crc(new), !matches!(op, Op::CoveredByte | Op::Checksum));
    if kind == 4 && op == Op::ScopeSubstitution {
        let &(offset, length, _) = records.iter().find(|r| r.2 == 3).unwrap();
        assert_eq!(length, 36);
        assert_eq!(new, &baseline[offset..offset + length], "exact lawful production donor");
        return;
    }
    assert_eq!(old.len(), new.len());
    let end = new.len() - 4;
    let field = match op {
        Op::CoveredByte => 16..17,
        Op::Checksum => end..end + 1,
        Op::Length => 12..16,
        Op::EnvelopeVersion => 8..9,
        Op::ScopeSubstitution if kind == 5 => 32..40,
        Op::ScopeSubstitution => 9..10,
        Op::SelectiveAggregate => match kind { 2 => 56..57, 4 => end - 1..end, 5 => 48..49, _ => unreachable!() },
        _ => unreachable!(),
    };
    for index in 0..new.len() {
        let can_reseal = index >= end && !matches!(op, Op::CoveredByte | Op::Checksum);
        if !field.contains(&index) && !can_reseal { assert_eq!(old[index], new[index], "unrelated selected-record byte {index}"); }
    }
    match op {
        Op::CoveredByte | Op::Checksum | Op::SelectiveAggregate => assert_eq!(new[field.start], old[field.start] ^ 1),
        Op::Length => assert_eq!(read32(new, 12), if kind == 4 {4097} else {(old.len() - 20) as u32 + 1}),
        Op::EnvelopeVersion => assert_eq!(new[8], 2),
        Op::ScopeSubstitution if kind == 5 => assert_eq!(read64(new, 32), read64(old, 32) + 1),
        Op::ScopeSubstitution => { assert_eq!(new[9], 4); assert!((1..=4096).contains(&(new.len() - 20))); }
        _ => unreachable!(),
    }
}

fn baseline_records(bytes: &[u8]) -> Vec<(usize, usize, u8)> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        assert_eq!(&bytes[offset..offset + 8], b"WCP7REC\0");
        assert_eq!(bytes[offset + 8], 1);
        let length = 20 + read32(bytes, offset + 12) as usize;
        assert!(valid_crc(&bytes[offset..offset + length]));
        records.push((offset, length, bytes[offset + 9]));
        offset += length;
    }
    assert_eq!(offset, bytes.len());
    assert_eq!(records.last().unwrap().1, 156);
    assert_eq!(records.last().unwrap().2, 5);
    records
}
fn read32(bytes: &[u8], offset: usize) -> u32 { u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) }
fn read64(bytes: &[u8], offset: usize) -> u64 { u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) }
fn valid_crc(frame: &[u8]) -> bool {
    let mut crc = u32::MAX;
    for &byte in &frame[..frame.len() - 4] {
        crc ^= u32::from(byte);
        for _ in 0..8 { crc = if crc & 1 == 1 { (crc >> 1) ^ 0x82f63b78 } else { crc >> 1 }; }
    }
    !crc == read32(frame, frame.len() - 4)
}
