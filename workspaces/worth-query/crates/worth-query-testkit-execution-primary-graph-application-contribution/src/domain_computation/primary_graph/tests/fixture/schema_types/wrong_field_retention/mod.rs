use worth_query_declaration::facade::application_aftermath::DeclaredAftermathPostcondition;
use worth_query_declaration::facade::application_aftermath::DeclaredApplicationAftermathContract;
use worth_query_declaration::facade::application_aftermath::DeclaredCorrectionMechanism;
use worth_query_declaration::facade::application_aftermath::DeclaredLoweringCorrespondenceRef;
use worth_query_declaration::facade::application_aftermath::DeclaredPreImageDemand;
use worth_query_declaration::facade::application_aftermath::DeclaredPreImageLocus;
use worth_query_declaration::facade::application_aftermath::DeclaredRecordedInverse;

pub fn aftermath() -> DeclaredApplicationAftermathContract<
    crate::domain_computation::primary_graph::tests::fixture::schema_types::IdentityExecutionSchema,
> {
    recorded_inverse("test-wrong-field-inverse", "test-wrong-field-lowering")
}

pub fn recorded_inverse(
    identity: &'static str,
    lowering: &'static str,
) -> DeclaredApplicationAftermathContract<
    crate::domain_computation::primary_graph::tests::fixture::schema_types::IdentityExecutionSchema,
> {
    DeclaredApplicationAftermathContract::runtime_alone(
        DeclaredCorrectionMechanism::RecordedInverse(
            DeclaredRecordedInverse::new(
                identity,
                DeclaredLoweringCorrespondenceRef::new(lowering).unwrap(),
                DeclaredAftermathPostcondition::ExactPriorTruth,
                DeclaredPreImageDemand::new(
                    [DeclaredPreImageLocus::from_field(
                        crate::domain_computation::primary_graph::tests::fixture::schema_types::AccountStatus::reference(),
                    )],
                    256,
                )
                .unwrap(),
            )
            .unwrap(),
        ),
    )
}
