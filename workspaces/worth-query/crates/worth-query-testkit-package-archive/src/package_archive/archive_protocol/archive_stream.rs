#[path = "archive_stream/fixture.rs"]
pub mod fixture;
#[path = "archive_stream/predecessor.rs"]
pub mod predecessor;

pub use std::ops::Range;

pub use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecordFamily as Family,
};
pub use worth_query_package_archive::facade::*;

pub const MANIFEST_FRAME_BYTES: usize = 118;
pub const RECORD_FRAME_HEADER_BYTES: usize = 12;
pub const RETAINED_VERSION_TWO_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2.hex");

pub const RETAINED_RECORD_SEVEN_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v7.hex");
pub const RETAINED_RECORD_EIGHT_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v8.hex");
pub const RETAINED_RECORD_NINE_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v9.hex");
pub const RETAINED_RECORD_TEN_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v10.hex");
pub const RETAINED_RECORD_TWELVE_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v12.hex");
pub const RETAINED_RECORD_THIRTEEN_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v13.hex");
pub const RETAINED_RECORD_FOURTEEN_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v14.hex");
pub const RETAINED_RECORD_SIXTEEN_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v16.hex");
pub const RETAINED_RECORD_SEVENTEEN_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v17.hex");
pub const VERSION_TWO_MINIMAL_ARCHIVE_HEX: &str =
    include_str!("archive_stream/archive_v2_record_v18.hex");

pub fn frame_ranges(bytes: &[u8]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = MANIFEST_FRAME_BYTES;
    while start < bytes.len() {
        let payload = u32::from_be_bytes(bytes[start + 8..start + 12].try_into().unwrap()) as usize;
        let end = start + RECORD_FRAME_HEADER_BYTES + payload;
        ranges.push(start..end);
        start = end;
    }
    ranges
}

pub fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

pub fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut encoded, byte| {
        write!(&mut encoded, "{byte:02x}").unwrap();
        encoded
    })
}

pub fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(digits, 16).unwrap()
        })
        .collect()
}

pub fn assert_decode_kind(bytes: &[u8], expected: WorthQueryPackageArchiveDenialKind) {
    assert_kind(
        decode_package_archive(bytes, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err(),
        expected,
    );
}

pub fn assert_kind(
    error: WorthQueryPackageArchiveDenial,
    expected: WorthQueryPackageArchiveDenialKind,
) {
    assert_eq!(error.kind(), expected);
}
