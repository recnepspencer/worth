#[allow(unused_imports)]
use crate::wal_frame_v1_golden::*;

#[test]
fn wal_frame_v1_matches_frozen_literal_and_independent_digests() {
    let expected_header = literal(HEADER_HEX);
    let expected_payload_sha = literal(PAYLOAD_SHA_HEX);
    let expected_frame = literal(FRAME_HEX);
    let expected_footer = literal(FOOTER_SHA_HEX);
    let payload = [0x10, 0x20, 0x30];

    assert_eq!(expected_header.len(), WAL_FRAME_V1_HEADER_BYTES);
    assert_eq!(expected_frame.len(), 151);
    assert_eq!(expected_footer.len(), WAL_FRAME_V1_FOOTER_BYTES);
    assert_eq!(
        independent_sha256(&payload).as_slice(),
        expected_payload_sha
    );
    assert_eq!(
        independent_sha256(&expected_frame[..WAL_FRAME_V1_HEADER_BYTES + payload.len()]).as_slice(),
        expected_footer
    );

    let encoded = encode_wal_frame_v1(
        worth_store_physical_format::wal_frame::WalFrameV1EncodeRequest::new(
            1,
            2,
            3,
            4,
            b"c9-wal-v1-golden",
            &payload,
        )
        .expect("valid frozen WAL request"),
    );
    assert_eq!(encoded, expected_frame);

    let header: &[u8; WAL_FRAME_V1_HEADER_BYTES] = expected_header
        .as_slice()
        .try_into()
        .expect("literal header width");
    let decoded = decode_wal_frame_v1_header(header).expect("literal header decodes");
    assert_eq!((decoded.segment_id(), decoded.generation()), (1, 2));
    assert_eq!((decoded.lsn_start(), decoded.lsn_end()), (3, 4));
    let mut calculator =
        worth_store_physical_format::wal_frame::WalFrameV1ChecksumCalculator::new(header);
    calculator.update_payload(&payload[..1]).unwrap();
    calculator.update_payload(&payload[1..]).unwrap();
    let footer: &[u8; 32] = expected_footer.as_slice().try_into().unwrap();
    calculator.finish(decoded, footer).unwrap();

    let bounded = decode_bounded_wal_frame_v1(&expected_frame).unwrap();
    assert_eq!(bounded.header(), decoded);
    assert_eq!(bounded.payload(), payload);
    assert_eq!(bounded.frame_digest().as_slice(), expected_footer);
}
