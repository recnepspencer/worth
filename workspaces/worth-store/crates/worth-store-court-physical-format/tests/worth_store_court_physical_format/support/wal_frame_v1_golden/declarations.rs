

pub(crate) use {crate::store_physical_format_support::independent_sha256};
pub(crate) use {worth_store_physical_format::wal_frame::decode_bounded_wal_frame_v1, worth_store_physical_format::wal_frame::decode_wal_frame_v1_header, worth_store_physical_format::wal_frame::encode_wal_frame_v1, worth_store_physical_format::wal_frame::WalFrameV1BoundedDecodeDenial, worth_store_physical_format::wal_frame::WalFrameV1ChecksumCalculator, worth_store_physical_format::wal_frame::WalFrameV1EncodeRequest, worth_store_physical_format::wal_frame::WAL_FRAME_V1_FOOTER_BYTES, worth_store_physical_format::wal_frame::WAL_FRAME_V1_HEADER_BYTES};

pub(crate) const HEADER_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9";
pub(crate) const PAYLOAD_SHA_HEX: &str = "8e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9";
pub(crate) const FRAME_HEX: &str = "574f52544857414c0100740001000000000000000200000000000000030000000000000004000000000000000300000000000000d675c4a7b3dc55cebb3c413a084473b6e80a549d48106af3439bbdf5c76eb5768e1336ab78ebe687fd8056a37f2d3b0c32f4cf8fa8b691b653800fa693d570b9102030c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";
pub(crate) const FOOTER_SHA_HEX: &str = "c8b5e38493d6397d83ab40999718d0fcaf7354454bff68c9a716003b636fa681";

pub(crate) fn literal(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("ASCII hex");
            u8::from_str_radix(text, 16).expect("literal hex byte")
        })
        .collect()
}