//! Exact-locus recorded-inverse fixtures for provider retention proofs.

use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract,
    DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand,
    DeclaredPreImageLocus, DeclaredReconciliationProcedure, DeclaredRecordedInverse,
};

pub(in super::super) fn status_with_external_owner(
) -> DeclaredApplicationAftermathContract<super::IdentityExecutionSchema> {
    DeclaredApplicationAftermathContract::runtime_with_external_owner(
        recorded_inverse(
            "test-status-external-inverse",
            "test-status-external-lowering",
            [
                DeclaredPreImageLocus::from_field(super::AccountStatus::at("AccountStatus"))
                    .expect("record field contract"),
            ],
        ),
        DeclaredReconciliationProcedure::new("confirm-test-status-effect").unwrap(),
    )
}

pub(in super::super) fn two_field_inverse(
) -> DeclaredApplicationAftermathContract<super::IdentityExecutionSchema> {
    DeclaredApplicationAftermathContract::runtime_alone(recorded_inverse(
        "test-two-field-inverse",
        "test-two-field-lowering",
        [
            DeclaredPreImageLocus::from_field(super::AccountStatus::at("AccountStatus"))
                .expect("record field contract"),
            DeclaredPreImageLocus::from_field(super::AccountLabel::at("AccountLabel"))
                .expect("record field contract"),
        ],
    ))
}

fn recorded_inverse(
    identity: &'static str,
    lowering: &'static str,
    loci: impl IntoIterator<Item = DeclaredPreImageLocus<super::IdentityExecutionSchema>>,
) -> DeclaredCorrectionMechanism<super::IdentityExecutionSchema> {
    DeclaredCorrectionMechanism::RecordedInverse(
        DeclaredRecordedInverse::new(
            identity,
            DeclaredLoweringCorrespondenceRef::new(lowering).unwrap(),
            DeclaredAftermathPostcondition::ExactPriorTruth,
            DeclaredPreImageDemand::new(loci, 512).unwrap(),
        )
        .unwrap(),
    )
}
