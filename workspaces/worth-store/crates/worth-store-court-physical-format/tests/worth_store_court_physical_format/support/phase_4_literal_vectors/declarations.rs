






pub(crate) fn checkpoint_stream() -> Vec<u8> {
    use {crate::phase_4_literal_vectors::checkpoint_records::BINDING, crate::phase_4_literal_vectors::checkpoint_records::BINDING_COMPACTION, crate::phase_4_literal_vectors::checkpoint_records::DIRTY_BASIS, crate::phase_4_literal_vectors::checkpoint_records::FOOTER, crate::phase_4_literal_vectors::checkpoint_records::HEADER};
    [HEADER, DIRTY_BASIS, BINDING_COMPACTION, BINDING, FOOTER]
        .into_iter()
        .flat_map(crate::phase_4_literal_vectors::oracle::decode_hex)
        .collect()
}

pub(crate) fn pending_operation() -> Vec<u8> {
    crate::phase_4_literal_vectors::oracle::decode_hex(crate::phase_4_literal_vectors::physical_work_obligation::OPERATION_3_HEX)
}

pub(crate) fn routing_block() -> Vec<u8> {
    crate::phase_4_literal_vectors::oracle::decode_hex(crate::phase_4_literal_vectors::durable_frames::ROOT_ROUTING)
}

pub(crate) fn bootstrap_catalog() -> Vec<u8> {
    crate::phase_4_literal_vectors::oracle::decode_hex(crate::phase_4_literal_vectors::durable_frames::BOOTSTRAP)
}

pub(crate) fn wal_range(segment: u64, start: u64, end: u64) -> Vec<u8> {
    let mut bytes = crate::phase_4_literal_vectors::oracle::decode_hex(crate::phase_4_literal_vectors::wal_frame::FRAME_HEX);
    bytes[12..20].copy_from_slice(&segment.to_le_bytes());
    bytes[28..36].copy_from_slice(&start.to_le_bytes());
    bytes[36..44].copy_from_slice(&end.to_le_bytes());
    let footer = bytes.len() - 32;
    let digest = crate::phase_4_literal_vectors::oracle::sha256(&[&bytes[..footer]]);
    bytes[footer..].copy_from_slice(&digest);
    bytes
}