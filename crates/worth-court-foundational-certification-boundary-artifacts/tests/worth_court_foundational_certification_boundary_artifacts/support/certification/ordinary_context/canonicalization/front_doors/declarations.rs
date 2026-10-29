pub(crate) use {worth_foundational::admit_authoritative_record_aspect_state, worth_foundational::canonicalization, worth_foundational::validate_aspect_value, worth_foundational::AspectLocator, worth_foundational::AspectValue, worth_foundational::AspectValueLocator, worth_foundational::BoundarySourceLocator, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalMilestone2PhaseGate, worth_foundational::CanonicalMismatchKind, worth_foundational::CanonicalProducerShape, worth_foundational::CanonicalizationRuleVersion, worth_foundational::LocatorAuthority, worth_foundational::ScalarAspectType};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::scalar_contract};

pub(crate) fn version() -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new("m2.dx.front-door").expect("valid version")
}

pub(crate) fn admitted_scalar_state(
    aspect_key: &str,
    aspect_identity: u64,
    value: i64,
) -> worth_foundational::AuthoritativeRecordAspectStateArtifact {
    let contract = scalar_contract(aspect_key, aspect_identity, worth_foundational::ScalarAspectType::Int64);
    let worth_proof::TransitionOutcome::Success(validated) =
        validate_aspect_value(&contract, worth_foundational::AspectValue::Int64(value).into())
    else {
        panic!("expected validation");
    };
    let worth_proof::TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state([validated])
    else {
        panic!("expected authoritative state");
    };
    state
}