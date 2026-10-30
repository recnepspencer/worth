pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::admit_authoritative_record_aspect_state,
    worth_foundational::canonicalization_api::common_path,
    worth_foundational::canonicalization_api::lower_lane::basis as lower_basis,
    worth_foundational::canonicalization_api::lower_lane::comparison as lower_comparison,
    worth_foundational::canonicalization_api::lower_lane::digest as lower_digest,
    worth_foundational::canonicalization_api::lower_lane::export as lower_export,
    worth_foundational::canonicalization_api::stronger_lane::readiness as stronger_readiness,
    worth_foundational::validate_aspect_value, worth_foundational::AspectValue,
    worth_foundational::CanonicalEquivalenceBasis,
    worth_foundational::CanonicalMilestone2PhaseGate, worth_foundational::CanonicalProducerShape,
    worth_foundational::ScalarAspectType,
};

pub(crate) use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::scalar_contract};

pub(crate) fn version(
) -> worth_foundational::canonicalization_api::lower_lane::basis::CanonicalizationRuleVersion {
    worth_foundational::canonicalization_api::lower_lane::basis::CanonicalizationRuleVersion::new(
        "m2.dx.grouped-surface",
    )
    .expect("valid version")
}

pub(crate) fn admitted_scalar_state(
    aspect_key: &str,
    aspect_identity: u64,
    value: i64,
) -> worth_foundational::AuthoritativeRecordAspectStateArtifact {
    let contract = scalar_contract(
        aspect_key,
        aspect_identity,
        worth_foundational::ScalarAspectType::Int64,
    );
    let worth_proof::TransitionOutcome::Success(validated) = validate_aspect_value(
        &contract,
        worth_foundational::AspectValue::Int64(value).into(),
    ) else {
        panic!("expected validation");
    };
    let worth_proof::TransitionOutcome::Success(state) =
        admit_authoritative_record_aspect_state([validated])
    else {
        panic!("expected authoritative state");
    };
    state
}
