#[allow(unused_imports)]
use crate::wal_frame_v1_golden::*;

#[test]
fn bounded_decode_withholds_fields_when_declared_checksum_coverage_fails() {
    let mut payload_damaged = literal(FRAME_HEX);
    payload_damaged[WAL_FRAME_V1_HEADER_BYTES] ^= 1;
    let denial = decode_bounded_wal_frame_v1(&payload_damaged).unwrap_err();
    let worth_store_physical_format::wal_frame::WalFrameV1BoundedDecodeDenial::ChecksumMismatch(
        mismatch,
    ) = denial
    else {
        panic!("covered payload mutation must fail checksum admission");
    };
    assert!(mismatch.payload_checksum());
    assert!(mismatch.frame_checksum());

    let complete = literal(FRAME_HEX);
    let truncated = &complete[..150];
    assert!(matches!(
        decode_bounded_wal_frame_v1(truncated),
        Err(worth_store_physical_format::wal_frame::WalFrameV1BoundedDecodeDenial::FrameLengthMismatch {
            declared: 151,
            observed: 150,
        })
    ));
}
