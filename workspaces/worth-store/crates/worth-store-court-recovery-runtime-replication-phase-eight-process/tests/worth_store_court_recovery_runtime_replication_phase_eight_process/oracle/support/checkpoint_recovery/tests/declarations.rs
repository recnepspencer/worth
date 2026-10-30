pub(crate) use {sha2::Digest, sha2::Sha256};

pub(crate) use {crate::history, crate::oracle::support::checkpoint_recovery::fate_from_files};

pub(crate) fn checkpoint_current(frontier: u64) -> Vec<u8> {
    checkpoint_current_schema(frontier, 1)
}

pub(crate) fn checkpoint_current_schema(frontier: u64, schema: u8) -> Vec<u8> {
    let mut header = vec![0; 144];
    header[..16].copy_from_slice(&[9; 16]);
    header[16..24].copy_from_slice(&1_u64.to_le_bytes());
    header[32..40].copy_from_slice(&frontier.to_le_bytes());
    header[64] = 1;
    let certified = schema == 3;
    let mut footer = vec![0; if certified { 184 } else { 136 }];
    footer[..24].copy_from_slice(&header[..24]);
    footer[32..64].copy_from_slice(&sha2::Sha256::digest([]));
    footer[64..72].copy_from_slice(&164_u64.to_le_bytes());
    footer[80..88].copy_from_slice(&frontier.to_le_bytes());
    footer[104..136].copy_from_slice(&sha2::Sha256::digest([]));
    let mut compaction = vec![0; 16];
    compaction[8..16].copy_from_slice(&frontier.to_le_bytes());
    let mut bytes = record(schema, 1, &header);
    bytes.extend_from_slice(&record(schema, 3, &compaction));
    if certified {
        // One release-custody certificate record precedes the footer.
        let certificate = record(schema, 7, &[7; 24]);
        footer[136..144].copy_from_slice(&1_u64.to_le_bytes());
        footer[144..152].copy_from_slice(&(certificate.len() as u64).to_le_bytes());
        footer[152..184].copy_from_slice(&sha2::Sha256::digest(&certificate));
        bytes.extend_from_slice(&certificate);
    }
    bytes.extend_from_slice(&record(schema, 5, &footer));
    bytes
}

pub(crate) fn record(schema: u8, kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"WCP7REC\0");
    bytes.push(schema);
    bytes.push(kind);
    bytes.extend_from_slice(&[0; 2]);
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&crc32c(&bytes).to_le_bytes());
    bytes
}

pub(crate) fn crc32c(bytes: &[u8]) -> u32 {
    let mut value = !0_u32;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(value & 1);
            value = (value >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !value
}

pub(crate) fn reseal_footer(wal: &mut Vec<u8>) {
    let footer = sha2::Sha256::digest(&wal);
    wal.extend_from_slice(&footer);
}

pub(crate) fn append_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
}

pub(crate) fn field(target: &mut Sha256, field: &[u8]) {
    target.update((field.len() as u64).to_le_bytes());
    target.update(field);
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
