use super::*;

#[test]
fn reserved_schema_ten_root_frame_binds_head_frontier_and_requires_full_backing() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let root = DurablePhysicalRootManifest::builder(8, 9, 2, 43)
        .next_release_custody_head_block(2)
        .admit()
        .unwrap()
        .with_maintenance_protocol();
    let required = root.encoded_frame_bytes();
    assert_eq!(required, 728);
    assert!(root
        .encode_in_reserved(format, Vec::with_capacity(required - 1))
        .is_none());
    let bytes = root
        .encode_in_reserved(format, Vec::with_capacity(required))
        .unwrap();
    assert_eq!(bytes.len(), required);
    assert_eq!(bytes[8], DurableFrameKind::RootManifest as u8);
    assert_eq!(bytes[9], 10);
    assert_eq!(
        DurablePhysicalRootManifest::decode(&bytes, 200),
        Ok((root, format))
    );
}
