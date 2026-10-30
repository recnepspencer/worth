pub use sha2::{Digest, Sha256};
pub use worth_query_package_archive::facade::*;

pub use super::archive_stream::fixture;
pub use super::release_envelope_fixture::*;

#[path = "release_envelope/golden.rs"]
pub mod golden;

pub const ENVELOPE_HEADER_BYTES: usize = 18;
pub const EXPECTED_IDENTITY_BYTES: usize = 32;
pub const CHECKSUM_BYTES: usize = 32;

pub fn archive_length(bytes: &[u8], archive_offset: usize) -> usize {
    let length_offset = archive_offset - 8;
    usize::try_from(u64::from_be_bytes(
        bytes[length_offset..archive_offset].try_into().unwrap(),
    ))
    .unwrap()
}

pub fn find_text(bytes: &[u8], needle: &[u8]) -> usize {
    bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .unwrap()
}

pub fn assert_decode_kind(
    bytes: &[u8],
    limits: WorthQueryPackageEnvelopeLimits,
    expected: WorthQueryPackageArchiveDenialKind,
) {
    assert_eq!(
        decode_package_release_envelope(bytes, limits)
            .unwrap_err()
            .kind(),
        expected
    );
}
