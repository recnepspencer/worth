use sha2::{Digest, Sha256};

use super::{observe, valid_dirty_basis};

fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}

fn record(schema: u8, kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(20 + payload.len());
    bytes.extend_from_slice(b"WCP7REC\0");
    bytes.extend_from_slice(&[schema, kind, 0, 0]);
    bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&crc32c(&bytes).to_le_bytes());
    bytes
}

fn replace_footer(stream: &mut Vec<u8>, schema: u8, change: impl FnOnce(&mut [u8])) {
    let footer_bytes = if schema == 3 { 204 } else { 156 };
    let offset = stream.len() - footer_bytes;
    let mut payload = stream[offset + 16..stream.len() - 4].to_vec();
    change(&mut payload);
    stream.truncate(offset);
    stream.extend(record(schema, 5, &payload));
}

fn replace_record_schema(stream: &mut [u8], offset: usize, frame_bytes: usize, schema: u8) {
    let record = &mut stream[offset..offset + frame_bytes];
    record[8] = schema;
    let checksum_offset = frame_bytes - 4;
    let checksum = crc32c(&record[..checksum_offset]);
    record[checksum_offset..].copy_from_slice(&checksum.to_le_bytes());
}

fn stream(schema: u8, dirty_kind: Option<u8>, certificates: &[(u8, &[u8])]) -> Vec<u8> {
    let mut header = [0_u8; 144];
    header[..16].copy_from_slice(&[1; 16]);
    header[16..24].copy_from_slice(&1_u64.to_le_bytes());
    header[24..32].copy_from_slice(&1_u64.to_le_bytes());
    header[32..40].copy_from_slice(&2_u64.to_le_bytes());
    header[40..48].copy_from_slice(&1_u64.to_le_bytes());
    header[48..56].copy_from_slice(&1_u64.to_le_bytes());
    header[64] = 1;
    let mut bytes = record(schema, 1, &header);

    let mut dirty = Vec::new();
    if let Some(kind) = dirty_kind {
        let mut payload = [0_u8; 48];
        payload[0] = kind;
        payload[8..16].copy_from_slice(&1_u64.to_le_bytes());
        payload[16..24].copy_from_slice(&1_u64.to_le_bytes());
        payload[32..36].copy_from_slice(&1_u32.to_le_bytes());
        dirty = record(schema, 2, &payload);
        bytes.extend_from_slice(&dirty);
    }
    let compaction_offset = u64::try_from(bytes.len()).unwrap();
    let mut compaction = [0_u8; 16];
    compaction[..8].copy_from_slice(&1_u64.to_le_bytes());
    compaction[8..16].copy_from_slice(&1_u64.to_le_bytes());
    bytes.extend(record(schema, 3, &compaction));

    let binding = record(schema, 4, &[0x5a]);
    bytes.extend_from_slice(&binding);
    let mut certificate_wire = Vec::new();
    for &(kind, payload) in certificates {
        certificate_wire.extend(record(schema, kind, payload));
    }
    bytes.extend_from_slice(&certificate_wire);

    let mut footer = vec![0_u8; if schema == 3 { 184 } else { 136 }];
    footer[..16].copy_from_slice(&[1; 16]);
    footer[16..24].copy_from_slice(&1_u64.to_le_bytes());
    footer[24..32].copy_from_slice(&u64::from(dirty_kind.is_some()).to_le_bytes());
    footer[32..64].copy_from_slice(&Sha256::digest(&dirty));
    footer[64..72].copy_from_slice(&compaction_offset.to_le_bytes());
    footer[72..80].copy_from_slice(&1_u64.to_le_bytes());
    footer[80..88].copy_from_slice(&1_u64.to_le_bytes());
    footer[88..96].copy_from_slice(&1_u64.to_le_bytes());
    footer[96..104].copy_from_slice(&u64::try_from(binding.len()).unwrap().to_le_bytes());
    footer[104..136].copy_from_slice(&Sha256::digest(&binding));
    if schema == 3 {
        footer[136..144].copy_from_slice(&u64::try_from(certificates.len()).unwrap().to_le_bytes());
        footer[144..152]
            .copy_from_slice(&u64::try_from(certificate_wire.len()).unwrap().to_le_bytes());
        footer[152..184].copy_from_slice(&Sha256::digest(&certificate_wire));
    }
    bytes.extend(record(schema, 5, &footer));
    bytes
}

#[test]
fn legacy_and_certified_checkpoint_envelopes_keep_independent_counts() {
    for schema in [1, 2] {
        let observed = observe(&stream(schema, None, &[])).unwrap();
        assert_eq!(observed.binding_records(), 1);
        assert_eq!(observed.dirty_records(), 0);
        assert_eq!(observed.durable_lsn(), 1);
    }
    let observed = observe(&stream(3, Some(17), &[(6, &[9]), (7, &[8])])).unwrap();
    assert_eq!(observed.dirty_records(), 1);
    assert_eq!(observed.binding_records(), 1);
    assert_eq!(observed.root_generation(), 1);
}

#[test]
fn certified_certificate_section_requires_crc_and_aggregate() {
    let original = stream(3, None, &[(6, &[9]), (7, &[8])]);
    let certificate_offset = 164 + 36 + 21;
    let mut damaged_crc = original.clone();
    damaged_crc[certificate_offset + 16] ^= 1;
    assert!(observe(&damaged_crc).is_none());

    let mut altered_with_repaired_crc = original;
    let record = &mut altered_with_repaired_crc[certificate_offset..certificate_offset + 21];
    record[16] ^= 1;
    let checksum = crc32c(&record[..17]);
    record[17..21].copy_from_slice(&checksum.to_le_bytes());
    assert!(observe(&altered_with_repaired_crc).is_none());
}

#[test]
fn certified_counts_bounds_and_exact_end_are_structural() {
    let certificates = [(7, &[8][..]); 65];
    assert!(observe(&stream(3, None, &certificates[..64])).is_some());
    assert!(observe(&stream(3, None, &certificates)).is_none());
    let base = stream(3, None, &[(7, &[8])]);
    for (offset, value) in [(136, 2_u64), (144, 22), (136, 65), (144, 65_537)] {
        let mut wrong = base.clone();
        replace_footer(&mut wrong, 3, |footer| {
            footer[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        });
        assert!(observe(&wrong).is_none());
    }
    let mut trailing = base.clone();
    trailing.insert(trailing.len() - 204, 0);
    assert!(observe(&trailing).is_none());
    let mut truncated = base;
    truncated.pop();
    assert!(observe(&truncated).is_none());
}

#[test]
fn maximum_certificate_payload_is_opaque_but_still_bounded() {
    assert!(observe(&stream(3, None, &[(7, &vec![1; 65_516])])).is_some());
    assert!(observe(&stream(3, None, &[(7, &vec![1; 65_517])])).is_none());
}

#[test]
fn certificate_order_kind_and_record_schema_are_exact() {
    for certificates in [
        vec![(7, &[8][..]), (6, &[9][..])],
        vec![(6, &[8][..]), (6, &[9][..])],
        vec![(8, &[8][..])],
    ] {
        assert!(observe(&stream(3, None, &certificates)).is_none());
    }
    let mut mixed = stream(3, None, &[(7, &[8])]);
    replace_record_schema(&mut mixed, 164, 36, 2);
    assert!(observe(&mixed).is_none());
    let mut mixed_certificate = stream(3, None, &[(7, &[8])]);
    let certificate_offset = 164 + 36 + 21;
    replace_record_schema(&mut mixed_certificate, certificate_offset, 21, 2);
    let digest = Sha256::digest(&mixed_certificate[certificate_offset..certificate_offset + 21]);
    replace_footer(&mut mixed_certificate, 3, |footer| {
        footer[152..184].copy_from_slice(&digest);
    });
    assert!(observe(&mixed_certificate).is_none());
    assert!(observe(&stream(4, None, &[])).is_none());
}

#[test]
fn dirty_basis_rejects_missing_head_coordinate_and_unknown_legacy_codes() {
    let mut payload = [0_u8; 48];
    payload[0] = 16;
    payload[8..16].copy_from_slice(&1_u64.to_le_bytes());
    payload[32..36].copy_from_slice(&1_u32.to_le_bytes());
    assert!(valid_dirty_basis(&payload).is_some());
    payload[16..24].copy_from_slice(&1_u64.to_le_bytes());
    assert!(valid_dirty_basis(&payload).is_none());
    payload[0] = 17;
    assert!(valid_dirty_basis(&payload).is_some());
    payload[16..24].fill(0);
    assert!(valid_dirty_basis(&payload).is_none());
    for unknown in [8, 9, 18] {
        payload[0] = unknown;
        assert!(valid_dirty_basis(&payload).is_none());
    }
}
