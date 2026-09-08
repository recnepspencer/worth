mod checkpoint_records;
mod durable_frames;
mod oracle;
mod page_frames;
mod physical_work_obligation;
mod wal_frame;

pub(crate) fn checkpoint_stream() -> Vec<u8> {
    use checkpoint_records::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER};
    [HEADER, DIRTY_BASIS, BINDING_COMPACTION, BINDING, FOOTER]
        .into_iter()
        .flat_map(oracle::decode_hex)
        .collect()
}

pub(crate) fn pending_operation() -> Vec<u8> {
    oracle::decode_hex(physical_work_obligation::OPERATION_3_HEX)
}

pub(crate) fn routing_block() -> Vec<u8> {
    oracle::decode_hex(durable_frames::ROOT_ROUTING)
}

pub(crate) fn bootstrap_catalog() -> Vec<u8> {
    oracle::decode_hex(durable_frames::BOOTSTRAP)
}

pub(crate) fn wal_range(segment: u64, start: u64, end: u64) -> Vec<u8> {
    let mut bytes = oracle::decode_hex(wal_frame::FRAME_HEX);
    bytes[12..20].copy_from_slice(&segment.to_le_bytes());
    bytes[28..36].copy_from_slice(&start.to_le_bytes());
    bytes[36..44].copy_from_slice(&end.to_le_bytes());
    let footer = bytes.len() - 32;
    let digest = oracle::sha256(&[&bytes[..footer]]);
    bytes[footer..].copy_from_slice(&digest);
    bytes
}

#[test]
fn physical_work_literal_is_independently_consumable() {
    physical_work_obligation::verify();
}

#[test]
fn durable_frame_family_literals_are_independently_consumable() {
    durable_frames::verify();
}

#[test]
fn every_declared_page_size_literal_is_independently_consumable() {
    page_frames::verify();
}

#[test]
fn wal_literal_is_independently_consumable() {
    wal_frame::verify();
}

#[test]
fn every_checkpoint_record_kind_is_independently_consumable() {
    checkpoint_records::verify();
}
