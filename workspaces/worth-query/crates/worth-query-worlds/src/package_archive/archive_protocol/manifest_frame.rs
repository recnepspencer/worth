pub use worth_query_installation::facade::{
    WorthQueryPortableDomainPackageIdentity, WorthQueryPortablePackageManifest,
    WorthQueryPortablePackageReconstruction, WorthQueryPortablePackageReconstructionLimits,
    WorthQueryPortablePackageRecordFamily, WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
};
pub use worth_query_package_archive::facade::*;

pub const CURRENT_MANIFEST_FRAME_HEX: &str = "5751504b47415200000200000068000141414141414141414141414141414141414141414141414141414141414141410000000100000000000000000000000000000040000c000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

pub fn manifest() -> WorthQueryPortablePackageManifest {
    let mut counts = [0; WorthQueryPortablePackageRecordFamily::ALL.len()];
    counts[0] = 1;
    WorthQueryPortablePackageManifest::from_untrusted_fields(
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes([0x41; 32]),
        1,
        0,
        64,
        counts,
    )
}

pub fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut encoded, byte| {
            write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
            encoded
        },
    )
}

pub fn decode_hex(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 2, 0, "golden hex must contain whole bytes");
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).expect("golden hex is ASCII");
            u8::from_str_radix(digits, 16).expect("golden hex contains valid digits")
        })
        .collect()
}
