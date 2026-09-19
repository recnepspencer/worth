use super::{map_basis_denial, map_registration_denial};
use crate::domain_computation::primary_graph::application_query::{
    resource_lifecycle::WorthQueryApplicationBasisRegistrationDenial,
    WorthQueryApplicationQueryAdmissionDenialKind,
};
use worth_relational::facade::branch::RelationalBranchBasisDenial;

#[test]
fn owner_unavailable_maps_to_basis_unavailable_with_exact_subject() {
    let denial = map_basis_denial(RelationalBranchBasisDenial::OwnerUnavailable);

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable
    );
    assert_eq!(denial.subject(), "OwnerUnavailable");
}

#[test]
fn distinct_basis_denial_classes_keep_their_query_meaning() {
    let cases = [
        (
            RelationalBranchBasisDenial::MalformedDescriptor,
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnsupported,
        ),
        (
            RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: 17,
                actual_runtime_instance_id: 23,
            },
            WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
        ),
        (
            RelationalBranchBasisDenial::StaleReferenceGeneration,
            WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
        ),
        (
            RelationalBranchBasisDenial::OwnerFailure,
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
        ),
    ];

    for (denial, expected_kind) in cases {
        assert_eq!(map_basis_denial(denial).kind(), expected_kind);
    }
}

#[test]
fn registration_basis_denial_delegates_owner_unavailable_mapping() {
    let denial = map_registration_denial(WorthQueryApplicationBasisRegistrationDenial::Basis(
        RelationalBranchBasisDenial::OwnerUnavailable,
    ));

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable
    );
    assert_eq!(denial.subject(), "OwnerUnavailable");
}
