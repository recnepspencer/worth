#![cfg_attr(test, allow(clippy::disallowed_methods))]
use worth_foundational::{
    aspects, AspectValue, ContractValidationInput, InternedString, ScalarAspectType,
};
use worth_proof::TransitionOutcome;
use worth_store::aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectIdentity,
    StorePhysicalAuthorityWitness, StorePhysicalBoundaryWitness,
    ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};

#[test]
fn downstream_code_authors_boundary_facts_through_public_native_facade() {
    let key = aspects()
        .vocabulary()
        .key("store.physical.segment.identity")
        .unwrap();
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(10))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String);
    let validated =
        match aspects()
            .validate()
            .against(&contract)
            .value(ContractValidationInput::from(AspectValue::String(
                InternedString::from("segment-001"),
            ))) {
            TransitionOutcome::Success(value) => value,
            outcome => panic!("validation should succeed: {outcome:?}"),
        };
    let state = match aspects().authoritative_state().admit([validated]) {
        TransitionOutcome::Success(state) => state,
        outcome => panic!("state admission should succeed: {outcome:?}"),
    };
    let physical_witness = StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap();

    let fact = StoreAspectBoundaryFact::from_admitted_state(
        StoreAspectIdentity::from_aspect_key(key.clone()),
        StoreAspectAuthorityInput::new(state, physical_witness),
    )
    .unwrap();

    assert_eq!(fact.identity(), &StoreAspectIdentity::from_aspect_key(key));
}

#[test]
fn downstream_code_can_classify_exact_residency_failure_reasons() {
    let _reason_projection: fn(
        worth_store::physical_runtime::PhysicalRecordResidencyFailure,
    ) -> worth_store::physical_runtime::PhysicalRecordResidencyFailureReason =
        worth_store::physical_runtime::PhysicalRecordResidencyFailure::reason;
    let _identity_reason =
        worth_store::physical_runtime::PhysicalRecordResidencyFailureReason::FrameIdentityOccupied;
    let _cardinality_reason =
        worth_store::physical_runtime::PhysicalRecordResidencyFailureReason::CandidateCardinalityMismatch {
            declared: 2,
            provided: 1,
        };
}
