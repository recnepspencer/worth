pub(crate) use {sha2::Digest, sha2::Sha256};

pub(crate) use {crate::phase_eight_process::history::parent_oracle::wire::checkpoint::crc32c, crate::phase_eight_process::history::parent_oracle::wire::checkpoint::observe_checkpoint, crate::phase_eight_process::history::parent_oracle::wire::checkpoint::validated_binding_payloads, crate::phase_eight_process::history::parent_oracle::wire::checkpoint::CHECKPOINT_PREFIX_BYTES};

pub(crate) fn stream(schema: u8, certificate_kinds: &[u8]) -> Vec<u8> {
    stream_with_binding(schema, certificate_kinds, None)
}

pub(crate) fn stream_with_binding(schema: u8, certificate_kinds: &[u8], binding: Option<&[u8]>) -> Vec<u8> {
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

    let mut certificate_digest = sha2::Sha256::new();
    let mut certificate_bytes = 0_u64;
    for (index, kind) in certificate_kinds.iter().copied().enumerate() {
        let encoded = record(3, kind, &[index as u8 + 1]);
        certificate_digest.update(&encoded);
        certificate_bytes += encoded.len() as u64;
        bytes.extend(encoded);
    }

    let mut footer = vec![0_u8; if schema == 3 { 184 } else { 136 }];
    footer[..24].copy_from_slice(&header[..24]);
    footer[32..64].copy_from_slice(&sha2::Sha256::digest([]));
    footer[64..72].copy_from_slice(&164_u64.to_le_bytes());
    footer[72..80].copy_from_slice(&5_u64.to_le_bytes());
    footer[80..88].copy_from_slice(&29_u64.to_le_bytes());
    if let Some(encoded) = &binding_record {
        footer[88..96].copy_from_slice(&1_u64.to_le_bytes());
        footer[96..104].copy_from_slice(&(encoded.len() as u64).to_le_bytes());
        footer[104..136].copy_from_slice(&sha2::Sha256::digest(encoded));
    } else {
        footer[104..136].copy_from_slice(&sha2::Sha256::digest([]));
    }
    if schema == 3 {
        footer[136..144].copy_from_slice(&(certificate_kinds.len() as u64).to_le_bytes());
        footer[144..152].copy_from_slice(&certificate_bytes.to_le_bytes());
        footer[152..184].copy_from_slice(&certificate_digest.finalize());
    }
    bytes.extend(record(schema, 5, &footer));
    bytes
}

pub(crate) fn record(schema: u8, kind: u8, payload: &[u8]) -> Vec<u8> {
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