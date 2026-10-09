use super::{checksum_parts, decode_frame, OfflineDurableManifestDenial, FRAME_HEADER_BYTES};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

#[test]
fn schema_three_is_admitted_only_for_root_and_routing_frames() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    assert!(decode_frame(&frame(8, 3, format), 8, format).is_ok());
    assert!(decode_frame(&frame(2, 3, format), 2, format).is_ok());
    assert_eq!(
        decode_frame(&frame(9, 3, format), 9, format).err(),
        Some(OfflineDurableManifestDenial::FrameDeclarationMismatch)
    );
    assert_eq!(
        decode_frame(&frame(8, 4, format), 8, format).err(),
        Some(OfflineDurableManifestDenial::FrameDeclarationMismatch)
    );
    assert!(decode_frame(&frame(8, 2, format), 8, format).is_ok());
}

fn frame(kind: u8, schema: u8, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
    let mut bytes = vec![0_u8; FRAME_HEADER_BYTES];
    bytes[..8].copy_from_slice(b"WRC5FRM\0");
    bytes[8] = kind;
    bytes[9] = schema;
    bytes[10..20].copy_from_slice(&format.canonical_identity_bytes());
    bytes[20..22].copy_from_slice(&(FRAME_HEADER_BYTES as u16).to_le_bytes());
    bytes[28..36].copy_from_slice(&1_u64.to_le_bytes());
    let checksum = checksum_parts(&[&bytes[..44]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
    bytes
}
