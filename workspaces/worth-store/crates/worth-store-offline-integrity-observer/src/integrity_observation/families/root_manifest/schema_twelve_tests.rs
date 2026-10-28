//! The independent reader accepts a root with the full schema-12 table.
use super::*;

#[test]
fn offline_observer_reads_schema_twelve_and_rejects_eleven_without_decoding() {
    let mut bytes = schema_twelve_wire_image();
    assert_eq!(bytes[9], 12);
    assert_eq!(bytes.len(), 3808);
    let mut counters = OfflineIntegrityObservationCounters::default();
    assert!(read_root_manifest(&bytes, &mut counters).is_ok());
    assert_eq!(counters.root_manifest_payload_decoder_entries, 1);
    bytes[9] = 11;
    let mut counters = OfflineIntegrityObservationCounters::default();
    assert!(matches!(
        read_root_manifest(&bytes, &mut counters),
        Err(OfflineIntegrityOutcome::Unsupported(_))
    ));
    assert_eq!(counters.root_manifest_payload_decoder_entries, 0);
}

/// An inert wire fixture for the independent parser, with every anchor slot
/// occupied. It uses neither the writer's encoder nor its root/anchor types.
/// The process courtroom separately observes real Store-published artifacts.
fn schema_twelve_wire_image() -> Vec<u8> {
    let mut bytes = vec![0; 3808];
    bytes[..8].copy_from_slice(b"WRC5FRM\0");
    bytes[8..12].copy_from_slice(&[2, 12, 2, 0]);
    bytes[12..16].copy_from_slice(&16_384_u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&[1, 1, 1, 24]);
    bytes[20..22].copy_from_slice(&48_u16.to_le_bytes());
    bytes[24..28].copy_from_slice(&3760_u32.to_le_bytes());
    bytes[28..36].copy_from_slice(&1_u64.to_le_bytes());
    let payload = &mut bytes[48..];
    payload[..8].copy_from_slice(&1_u64.to_le_bytes());
    payload[8..16].copy_from_slice(&9_u64.to_le_bytes());
    payload[16..18].copy_from_slice(&2_u16.to_le_bytes());
    payload[24..32].copy_from_slice(&32_u64.to_le_bytes());
    payload[32..40].copy_from_slice(&2_u64.to_le_bytes());
    payload[40] = 1;
    let routing = &mut payload[48..120];
    routing[..8].copy_from_slice(&1_u64.to_le_bytes());
    routing[8..16].copy_from_slice(&1_u64.to_le_bytes());
    routing[16..18].copy_from_slice(&4_u16.to_le_bytes());
    routing[20..24].copy_from_slice(&31_u32.to_le_bytes());
    write_record(&mut routing[24..48], 1);
    write_record(&mut routing[48..72], 32);
    payload[152..156].copy_from_slice(&43_u32.to_le_bytes());
    payload[224..232].copy_from_slice(&1_u64.to_le_bytes());
    payload[568..576].copy_from_slice(&1_u64.to_le_bytes());
    payload[680] = 32;
    for index in 0..32 {
        let entry = &mut payload[688 + index * 96..][..96];
        let name = format!("owner.{index:02}");
        entry[..name.len()].copy_from_slice(name.as_bytes());
        write_record(&mut entry[32..56], index as u64 + 1);
        entry[56..88].fill(index as u8);
        entry[88..96].copy_from_slice(&1_u64.to_le_bytes());
    }
    let checksum = crate::integrity_observation::crc32c::crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
    bytes
}

fn write_record(bytes: &mut [u8], ordinal: u64) {
    bytes[..16].fill(7);
    bytes[16..24].copy_from_slice(&ordinal.to_le_bytes());
}
