#[path = "domain_operation_record/fixture.rs"]
pub mod fixture;

pub use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecordFamily as Family,
};
pub use worth_query_package_archive::facade::*;

pub use fixture::{collection_operation_package, operation_package};

pub fn decode_hex(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 2, 0);
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
