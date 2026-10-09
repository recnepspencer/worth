use sha2::{Digest, Sha256};

use super::{crc32c, observe_checkpoint, validated_binding_payloads, CHECKPOINT_PREFIX_BYTES};

#[test]
fn certified_envelope_and_legacy_envelopes_remain_observable() {
    for schema in [1, 2, 3] {
        let certificates = if schema == 3 { &[6, 7][..] } else { &[][..] };
        let bytes = stream(schema, certificates);
        let facts = observe_checkpoint(&bytes).expect("lawful checkpoint envelope");
        assert_eq!(facts.sequence, 1);
        assert_eq!(facts.covered, (11, 29));
        assert_eq!(facts.durable, 29);
    }
}

#[test]
fn certified_certificate_crc_and_aggregate_are_independent_checks() {
    let mut bytes = stream(3, &[6, 7]);
    let first_certificate = 164 + 36;
    bytes[first_certificate + CHECKPOINT_PREFIX_BYTES] ^= 1;
    assert!(observe_checkpoint(&bytes).is_none(), "certificate CRC");

    let payload_bytes = u32::from_le_bytes(
        bytes[first_certificate + 12..first_certificate + 16]
            .try_into()
            .unwrap(),
    ) as usize;
    let checksum = crc32c(
        &bytes[first_certificate..first_certificate + CHECKPOINT_PREFIX_BYTES + payload_bytes],
    );
    bytes[first_certificate + CHECKPOINT_PREFIX_BYTES + payload_bytes
        ..first_certificate + CHECKPOINT_PREFIX_BYTES + payload_bytes + 4]
        .copy_from_slice(&checksum.to_le_bytes());
    assert!(
        observe_checkpoint(&bytes).is_none(),
        "valid record CRC cannot override the footer certificate aggregate"
    );
}

#[test]
fn mixed_record_schema_and_misordered_tier_are_not_selected() {
    let mut mixed = stream(3, &[6, 7]);
    let compaction = 164;
    mixed[compaction + 8] = 1;
    let checksum = crc32c(&mixed[compaction..compaction + 16 + 16]);
    mixed[compaction + 32..compaction + 36].copy_from_slice(&checksum.to_le_bytes());
    assert!(observe_checkpoint(&mixed).is_none());

    assert!(observe_checkpoint(&stream(3, &[7, 6])).is_none());
    assert!(observe_checkpoint(&stream(3, &[6, 6])).is_none());
}

#[test]
fn certified_envelope_exposes_only_aggregate_bound_binding_payloads() {
    let binding = b"independent-kind-four-binding";
    let bytes = stream_with_binding(3, &[6, 7], Some(binding));
    assert!(observe_checkpoint(&bytes).is_some());
    assert_eq!(validated_binding_payloads(&bytes), Some(vec![&binding[..]]));
}

fn stream(schema: u8, certificate_kinds: &[u8]) -> Vec<u8> {
    stream_with_binding(schema, certificate_kinds, None)
}

fn stream_with_binding(schema: u8, certificate_kinds: &[u8], binding: Option<&[u8]>) -> Vec<u8> {
    let mut header = [0_u8; 144];
    header[..16].fill(3);
    header[16..24].copy_from_slice(&1_u64.to_le_bytes());
    header[24..32].copy_from_slice(&11_u64.to_le_bytes());
    header[32..40].copy_from_slice(&29_u64.to_le_bytes());
    header[40..48].copy_from_slice(&5_u64.to_le_bytes());
    header[48..56].copy_from_slice(&71_u64.to_le_bytes());
    header[56..64].copy_from_slice(&43_u64.to_le_bytes());
    header[64] = 1;
    let mut bytes = record(schema, 1, &header);

    let mut compaction = [0_u8; 16];
    compaction[..8].copy_from_slice(&5_u64.to_le_bytes());
    compaction[8..16].copy_from_slice(&29_u64.to_le_bytes());
    bytes.extend(record(schema, 3, &compaction));

    let binding_record = binding.map(|payload| record(schema, 4, payload));
    if let Some(encoded) = &binding_record {
        bytes.extend_from_slice(encoded);
    }

    let mut certificate_digest = Sha256::new();
    let mut certificate_bytes = 0_u64;
    for (index, kind) in certificate_kinds.iter().copied().enumerate() {
        let encoded = record(3, kind, &[index as u8 + 1]);
        certificate_digest.update(&encoded);
        certificate_bytes += encoded.len() as u64;
        bytes.extend(encoded);
    }

    let mut footer = vec![0_u8; if schema == 3 { 184 } else { 136 }];
    footer[..24].copy_from_slice(&header[..24]);
    footer[32..64].copy_from_slice(&Sha256::digest([]));
    footer[64..72].copy_from_slice(&164_u64.to_le_bytes());
    footer[72..80].copy_from_slice(&5_u64.to_le_bytes());
    footer[80..88].copy_from_slice(&29_u64.to_le_bytes());
    if let Some(encoded) = &binding_record {
        footer[88..96].copy_from_slice(&1_u64.to_le_bytes());
        footer[96..104].copy_from_slice(&(encoded.len() as u64).to_le_bytes());
        footer[104..136].copy_from_slice(&Sha256::digest(encoded));
    } else {
        footer[104..136].copy_from_slice(&Sha256::digest([]));
    }
    if schema == 3 {
        footer[136..144].copy_from_slice(&(certificate_kinds.len() as u64).to_le_bytes());
        footer[144..152].copy_from_slice(&certificate_bytes.to_le_bytes());
        footer[152..184].copy_from_slice(&certificate_digest.finalize());
    }
    bytes.extend(record(schema, 5, &footer));
    bytes
}

fn record(schema: u8, kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0_u8; CHECKPOINT_PREFIX_BYTES + payload.len() + 4];
    bytes[..8].copy_from_slice(b"WCP7REC\0");
    bytes[8] = schema;
    bytes[9] = kind;
    bytes[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes[16..16 + payload.len()].copy_from_slice(payload);
    let checksum = crc32c(&bytes[..16 + payload.len()]);
    bytes[16 + payload.len()..].copy_from_slice(&checksum.to_le_bytes());
    bytes
}
