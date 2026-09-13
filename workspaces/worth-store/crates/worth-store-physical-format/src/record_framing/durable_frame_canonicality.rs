use super::durable_frame::{decode_frame, DurableFrameDenial, DurableFrameKind};

#[test]
fn non_data_families_reject_resealed_page_lsn_without_rejecting_data_lsn() {
    for kind in [
        DurableFrameKind::BootstrapCatalog,
        DurableFrameKind::RootManifest,
        DurableFrameKind::SegmentManifest,
        DurableFrameKind::ExtentManifest,
        DurableFrameKind::FreeSpaceManifest,
        DurableFrameKind::RootRoutingBlock,
        DurableFrameKind::SegmentMembershipBlock,
        DurableFrameKind::FreeSpaceMembershipBlock,
        DurableFrameKind::RootSelector,
    ] {
        let mut bytes = literal_envelope(kind);
        assert!(decode_frame(&bytes, kind).is_ok());
        for offset in 36..44 {
            bytes[36..44].fill(0);
            bytes[offset] = 1;
            reseal(&mut bytes);
            assert!(
                matches!(
                    decode_frame(&bytes, kind),
                    Err(DurableFrameDenial::NonDataPageLsnNonZero)
                ),
                "{kind:?} byte {offset}"
            );
        }
    }
    for kind in [DurableFrameKind::InlinePage, DurableFrameKind::Extent] {
        let mut bytes = literal_envelope(kind);
        bytes[36..44].copy_from_slice(&0x0807_0605_0403_0201_u64.to_le_bytes());
        reseal(&mut bytes);
        let (_, frame) = decode_frame(&bytes, kind).unwrap();
        assert_eq!(frame.page_lsn.get(), 0x0807_0605_0403_0201);
    }
}

#[test]
fn stale_checksum_still_blames_checksum_before_non_data_lsn_semantics() {
    let mut bytes = literal_envelope(DurableFrameKind::FreeSpaceManifest);
    bytes[36] = 1;
    assert!(matches!(
        decode_frame(&bytes, DurableFrameKind::FreeSpaceManifest),
        Err(DurableFrameDenial::IntegrityMismatch)
    ));
}

// This literal declares only the envelope under test; payload interpretation belongs
// to each family's decoder. No production encoder or checksum routine supplies it.
fn literal_envelope(kind: DurableFrameKind) -> Vec<u8> {
    let mut bytes = vec![0; 56];
    bytes[..8].copy_from_slice(b"WRC5FRM\0");
    bytes[8] = kind as u8;
    bytes[9] = 2;
    bytes[10..20].copy_from_slice(&[1, 0, 0, 64, 0, 0, 1, 1, 1, 24]);
    bytes[20] = 48;
    bytes[24] = 8;
    bytes[28] = 1;
    bytes[48..].fill(7);
    reseal(&mut bytes);
    bytes
}

fn reseal(bytes: &mut [u8]) {
    let mut crc = !0_u32;
    for byte in bytes[..44].iter().chain(bytes[48..].iter()) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    bytes[44..48].copy_from_slice(&(!crc).to_le_bytes());
}
