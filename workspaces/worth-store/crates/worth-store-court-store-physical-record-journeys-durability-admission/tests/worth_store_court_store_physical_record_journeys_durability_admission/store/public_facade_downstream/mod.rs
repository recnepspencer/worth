use {worth_foundational::aspects, worth_foundational::AspectValue, worth_foundational::ContractValidationInput, worth_foundational::InternedString, worth_foundational::ScalarAspectType};
use {worth_proof::TransitionOutcome};
use {worth_store::aspect_native::StoreAspectAuthorityInput, worth_store::aspect_native::StoreAspectBoundaryFact, worth_store::aspect_native::StoreAspectIdentity, worth_store::aspect_native::StorePhysicalAuthorityWitness, worth_store::aspect_native::StorePhysicalBoundaryWitness, worth_store::aspect_native::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};

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
        .scalar(worth_foundational::ScalarAspectType::String);
    let validated =
        match aspects()
            .validate()
            .against(&contract)
            .value(worth_foundational::ContractValidationInput::from(worth_foundational::AspectValue::String(
                worth_foundational::InternedString::from("segment-001"),
            ))) {
            worth_proof::TransitionOutcome::Success(value) => value,
            outcome => panic!("validation should succeed: {outcome:?}"),
        };
    let state = match aspects().authoritative_state().admit([validated]) {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state admission should succeed: {outcome:?}"),
    };
    let physical_witness = worth_store::aspect_native::StorePhysicalBoundaryWitness::from_physical_authority(
        worth_store::aspect_native::StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap();

    let fact = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        worth_store::aspect_native::StoreAspectIdentity::from_aspect_key(key.clone()),
        worth_store::aspect_native::StoreAspectAuthorityInput::new(state, physical_witness),
    )
    .unwrap();

    assert_eq!(fact.identity(), &worth_store::aspect_native::StoreAspectIdentity::from_aspect_key(key));
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
