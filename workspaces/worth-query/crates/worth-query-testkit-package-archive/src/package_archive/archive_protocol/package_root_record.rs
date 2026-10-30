pub use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDefinition,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryPortablePackageReconstruction, WorthQueryPortablePackageReconstructionDenial,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecordFamily as Family,
    WorthQueryValidatedPortableDomainPackage,
};
pub use worth_query_package_archive::facade::*;

#[path = "package_root_record/golden.rs"]
pub mod golden;
pub use golden::*;

pub fn root_package() -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "acme.workflow",
        3,
        7,
    ))
    .requires_capability("graph.read")
    .requires_configuration("workflow.runtime")
    .requires_operating_posture("durable")
    .definition(WorthQueryPortableDefinition::invariant(
        "balanced",
        "debits-equal-credits",
    ))
    .definition(WorthQueryPortableDefinition::graph_read_operation(
        "account-by-id",
        "exact-entity-read",
    ))
    .definition(WorthQueryPortableDefinition::declaration_family(
        "payment-notice",
        "external-effect-family",
    ))
    .permits_contribution("audit")
    .validate()
    .unwrap()
}

pub type Kind = WorthQueryPackageArchiveDenialKind;

pub fn assert_mutation_denial(bytes: &[u8], offset: usize, value: u8, expected: Kind) {
    let mut mutated = bytes.to_vec();
    mutated[offset] = value;
    assert_eq!(
        decode_package_root_record_frame(&mutated, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        expected
    );
}

pub fn encode_package_root_record_frame(
    view: worth_query_installation::facade::WorthQueryPortablePackageRecordView<'_>,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<Vec<u8>, WorthQueryPackageArchiveDenial> {
    encode_record_frame(view, limits)
}

pub fn decode_package_root_record_frame(
    bytes: &[u8],
    limits: WorthQueryPackageArchiveLimits,
) -> Result<WorthQueryUntrustedPortablePackageRecordFrame, WorthQueryPackageArchiveDenial> {
    WorthQueryPackageArchiveRecordDecoder::new(limits).decode_frame(bytes)
}
