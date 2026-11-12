use super::{contract, contract_with_resource_relation_name, Axis};
use crate::application_capability::{
    canonical_basis::prepare_capability_basis,
    WorthQueryApplicationCapabilityInstallationDenialKind,
};

#[test]
fn capability_canonical_bytes_are_bounded_before_identity_derivation() {
    let long_name = "x".repeat(192 * 1_024);
    let source_bytes = long_name.len();
    let contract = contract_with_resource_relation_name(Box::leak(long_name.into_boxed_str()));
    let package = worth_foundational::facade::CanonicalDigestId::new([1; 32]);
    let schema = worth_foundational::facade::CanonicalDigestId::new([2; 32]);
    let denial = prepare_capability_basis(&package, &schema, &contract).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalByteLimitExceeded
    );
    let admitted = prepare_capability_basis(&package, &schema, &super::contract(None)).unwrap();
    let Some(
        worth_foundational::facade::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
            maximum,
            attempted,
        },
    ) = denial.canonical_denial()
    else {
        panic!("real byte exhaustion must carry its canonical counts: {denial:?}");
    };
    assert_eq!(*maximum, admitted.maximum_canonical_encoded_bytes());
    assert!(*attempted > *maximum && *attempted >= source_bytes);
}

#[test]
fn capability_canonical_entries_are_bounded_before_identity_derivation() {
    let contract = contract(Some(Axis::OversizedComposition));
    let package = worth_foundational::facade::CanonicalDigestId::new([1; 32]);
    let schema = worth_foundational::facade::CanonicalDigestId::new([2; 32]);
    let denial = prepare_capability_basis(&package, &schema, &contract).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalEntryLimitExceeded
    );
    let admitted = prepare_capability_basis(&package, &schema, &super::contract(None)).unwrap();
    let Some(worth_foundational::facade::CanonicalDigestDerivationDenial::EntryLimitExceeded {
        maximum,
        actual,
    }) = denial.canonical_denial()
    else {
        panic!("real entry exhaustion must carry its canonical counts: {denial:?}");
    };
    assert_eq!(*maximum, admitted.maximum_canonical_entry_count());
    assert!(*actual > *maximum);
}

pub(crate) fn over_limit_denials() -> [(
    WorthQueryApplicationCapabilityInstallationDenialKind,
    crate::application_capability::WorthQueryApplicationCapabilityInstallationDenial,
); 2] {
    let package = worth_foundational::facade::CanonicalDigestId::new([1; 32]);
    let schema = worth_foundational::facade::CanonicalDigestId::new([2; 32]);
    let long_name = Box::leak("x".repeat(192 * 1_024).into_boxed_str());
    [
        (
            WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalByteLimitExceeded,
            contract_with_resource_relation_name(long_name),
        ),
        (
            WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalEntryLimitExceeded,
            contract(Some(Axis::OversizedComposition)),
        ),
    ]
    .map(|(kind, contract)| {
        (
            kind,
            prepare_capability_basis(&package, &schema, &contract).unwrap_err(),
        )
    })
}
