//! Exact-locus recorded-inverse fixtures for provider retention proofs.

use worth_query_declaration::facade::application_aftermath::DeclaredAftermathPostcondition;
use worth_query_declaration::facade::application_aftermath::DeclaredApplicationAftermathContract;
use worth_query_declaration::facade::application_aftermath::DeclaredCorrectionMechanism;
use worth_query_declaration::facade::application_aftermath::DeclaredLoweringCorrespondenceRef;
use worth_query_declaration::facade::application_aftermath::DeclaredPreImageDemand;
use worth_query_declaration::facade::application_aftermath::DeclaredPreImageLocus;
use worth_query_declaration::facade::application_aftermath::DeclaredReconciliationProcedure;
use worth_query_declaration::facade::application_aftermath::DeclaredRecordedInverse;

pub fn status_with_external_owner() -> DeclaredApplicationAftermathContract<
    crate::domain_computation::primary_graph::tests::fixture::schema_types::IdentityExecutionSchema,
> {
    DeclaredApplicationAftermathContract::runtime_with_external_owner(
        recorded_inverse(
            "test-status-external-inverse",
            "test-status-external-lowering",
            [DeclaredPreImageLocus::from_field(
                crate::domain_computation::primary_graph::tests::fixture::schema_types::AccountStatus::at("AccountStatus"),
            )],
        ),
        DeclaredReconciliationProcedure::new("confirm-test-status-effect"),
    )
}

pub fn two_field_inverse() -> DeclaredApplicationAftermathContract<
    crate::domain_computation::primary_graph::tests::fixture::schema_types::IdentityExecutionSchema,
> {
    DeclaredApplicationAftermathContract::runtime_alone(recorded_inverse(
        "test-two-field-inverse",
        "test-two-field-lowering",
        [
            DeclaredPreImageLocus::from_field(crate::domain_computation::primary_graph::tests::fixture::schema_types::AccountStatus::at("AccountStatus")),
            DeclaredPreImageLocus::from_field(crate::domain_computation::primary_graph::tests::fixture::schema_types::AccountLabel::at("AccountLabel")),
        ],
    ))
}

pub fn recorded_inverse(
    identity: &'static str,
    lowering: &'static str,
    loci: impl IntoIterator<Item = DeclaredPreImageLocus<crate::domain_computation::primary_graph::tests::fixture::schema_types::IdentityExecutionSchema>>,
) -> DeclaredCorrectionMechanism<
    crate::domain_computation::primary_graph::tests::fixture::schema_types::IdentityExecutionSchema,
> {
    DeclaredCorrectionMechanism::RecordedInverse(DeclaredRecordedInverse::new(
        identity,
        DeclaredLoweringCorrespondenceRef::new(lowering),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        DeclaredPreImageDemand::new(loci, 512),
    ))
}
