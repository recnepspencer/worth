pub use worth_query_package_archive::facade::*;

pub use super::archive_stream::fixture;
pub use super::release_envelope_fixture::{
    fixture_descriptor, signed_envelope_bytes, unsigned_envelope,
};

pub fn assert_kind(
    bytes: &[u8],
    limits: WorthQueryPackageEnvelopeLimits,
    expected: WorthQueryPackageArchiveDenialKind,
) {
    assert_eq!(
        decode_package_release_signing_payload(bytes, limits)
            .unwrap_err()
            .kind(),
        expected
    );
}
