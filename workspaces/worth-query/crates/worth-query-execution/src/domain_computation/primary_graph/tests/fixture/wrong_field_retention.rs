use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract,
    DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand,
    DeclaredPreImageLocus, DeclaredRecordedInverse,
};

pub(in super::super) fn aftermath(
) -> DeclaredApplicationAftermathContract<super::IdentityExecutionSchema> {
    recorded_inverse("test-wrong-field-inverse", "test-wrong-field-lowering")
}

fn recorded_inverse(
    identity: &'static str,
    lowering: &'static str,
) -> DeclaredApplicationAftermathContract<super::IdentityExecutionSchema> {
    DeclaredApplicationAftermathContract::runtime_alone(
        DeclaredCorrectionMechanism::RecordedInverse(
            DeclaredRecordedInverse::new(
                identity,
                DeclaredLoweringCorrespondenceRef::new(lowering).unwrap(),
                DeclaredAftermathPostcondition::ExactPriorTruth,
                DeclaredPreImageDemand::new(
                    [
                        DeclaredPreImageLocus::from_field(super::AccountStatus::at(
                            "AccountStatus",
                        ))
                        .expect("record field contract"),
                    ],
                    256,
                )
                .unwrap(),
            )
            .unwrap(),
        ),
    )
}
