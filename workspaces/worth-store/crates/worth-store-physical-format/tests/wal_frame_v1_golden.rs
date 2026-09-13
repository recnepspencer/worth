mod support;

use support::independent_sha256;
use worth_store_physical_format::wal_frame::{
    decode_bounded_wal_frame_v1, decode_wal_frame_v1_header, encode_wal_frame_v1,
    WalFrameV1BoundedDecodeDenial, WalFrameV1ChecksumCalculator, WalFrameV1EncodeRequest,
    WAL_FRAME_V1_FOOTER_BYTES, WAL_FRAME_V1_HEADER_BYTES,
};

const HEADER_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9";
const PAYLOAD_SHA_HEX: &str = "8e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9";
const FRAME_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";
const FOOTER_SHA_HEX: &str = "c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";

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
        WalFrameV1EncodeRequest::new(1, 2, 3, 4, b"c9-wal-v1-golden", &payload)
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
    let mut calculator = WalFrameV1ChecksumCalculator::new(header);
    calculator.update_payload(&payload[..1]).unwrap();
    calculator.update_payload(&payload[1..]).unwrap();
    let footer: &[u8; 32] = expected_footer.as_slice().try_into().unwrap();
    calculator.finish(decoded, footer).unwrap();

    let bounded = decode_bounded_wal_frame_v1(&expected_frame).unwrap();
    assert_eq!(bounded.header(), decoded);
    assert_eq!(bounded.payload(), payload);
    assert_eq!(bounded.frame_digest().as_slice(), expected_footer);
}

#[test]
fn bounded_decode_withholds_fields_when_declared_checksum_coverage_fails() {
    let mut payload_damaged = literal(FRAME_HEX);
    payload_damaged[WAL_FRAME_V1_HEADER_BYTES] ^= 1;
    let denial = decode_bounded_wal_frame_v1(&payload_damaged).unwrap_err();
    let WalFrameV1BoundedDecodeDenial::ChecksumMismatch(mismatch) = denial else {
        panic!("covered payload mutation must fail checksum admission");
    };
    assert!(mismatch.payload_checksum());
    assert!(mismatch.frame_checksum());

    let complete = literal(FRAME_HEX);
    let truncated = &complete[..150];
    assert!(matches!(
        decode_bounded_wal_frame_v1(truncated),
        Err(WalFrameV1BoundedDecodeDenial::FrameLengthMismatch {
            declared: 151,
            observed: 150,
        })
    ));
}

fn literal(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("ASCII hex");
            u8::from_str_radix(text, 16).expect("literal hex byte")
        })
        .collect()
}
