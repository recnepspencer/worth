pub use {worth_foundational::aspects, worth_foundational::AspectMask, worth_foundational::AspectValue, worth_foundational::ContractValidationInput, worth_foundational::InternedString, worth_foundational::MutationMask, worth_foundational::ProjectionMask, worth_foundational::ScalarAspectType};
pub use {worth_proof::TransitionOutcome};
pub use {worth_store::aspect_native::StoreAspectContractAdmission, worth_store::aspect_native::StoreAspectIdentity, worth_store::aspect_native::StorePhysicalAuthorityWitness, worth_store::aspect_native::StorePhysicalBoundaryWitness, worth_store::aspect_native::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
pub use {worth_store_contracts::ROADMAP_2_REPLAY_PHYSICAL_BOUNDARY};
pub use {worth_store_security::StoreAuthorityBoundSecurityScopeReceipt};

pub fn admitted_contract(
    revision: u64,
) -> (
    worth_foundational::AspectContract,
    StoreAspectIdentity,
    StoreAspectContractAdmission,
    StorePhysicalBoundaryWitness,
) {
    admitted_named_contract("store.physical.work.lifecycle", 71, revision)
}

pub fn admitted_named_contract(
    key: &str,
    identity_value: u64,
    revision: u64,
) -> (
    worth_foundational::AspectContract,
    StoreAspectIdentity,
    StoreAspectContractAdmission,
    StorePhysicalBoundaryWitness,
) {
    let key = aspects().vocabulary().key(key).unwrap();
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(aspects().vocabulary().identity(identity_value))
        .at_revision(aspects().vocabulary().revision(revision))
        .scalar(worth_foundational::ScalarAspectType::String);
    let witness = worth_store::aspect_native::StorePhysicalBoundaryWitness::from_physical_authority(
        worth_store::aspect_native::StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap();
    let identity = worth_store::aspect_native::StoreAspectIdentity::from_aspect_key(key);
    let admission = worth_store::aspect_native::StoreAspectContractAdmission::new(identity.clone(), contract.clone(), witness)
        .unwrap()
        .admit_projection_mask(AspectMask::<ProjectionMask>::whole_aspect())
        .unwrap()
        .admit_mutation_mask(AspectMask::<MutationMask>::whole_aspect())
        .unwrap();
    (contract, identity, admission, witness)
}

pub fn alternative_physical_witness() -> StorePhysicalBoundaryWitness {
    worth_store::aspect_native::StorePhysicalBoundaryWitness::from_physical_authority(
        worth_store::aspect_native::StorePhysicalAuthorityWitness::for_aspect_native_boundary_instance(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
            ROADMAP_2_REPLAY_PHYSICAL_BOUNDARY,
        )
        .unwrap(),
    )
    .unwrap()
}

pub fn security_scope(
    witness: StorePhysicalBoundaryWitness,
) -> StoreAuthorityBoundSecurityScopeReceipt {
    worth_store_security::admitted_store_internal_security_scope_for_physical_witness_test(witness)
        .authority_bound_receipt()
}

pub fn security_scope_from_authority(
    authority_key: &str,
    witness: StorePhysicalBoundaryWitness,
) -> StoreAuthorityBoundSecurityScopeReceipt {
    worth_store_security::admitted_store_internal_security_scope_for_named_physical_witness_test(
        authority_key,
        witness,
    )
    .authority_bound_receipt()
}

pub fn validated_value(
    contract: &worth_foundational::AspectContract,
    value: &str,
) -> worth_foundational::ContractValidatedAspectArtifact {
    match aspects()
        .validate()
        .against(contract)
        .value(worth_foundational::ContractValidationInput::from(worth_foundational::AspectValue::String(
            worth_foundational::InternedString::from(value),
        ))) {
        worth_proof::TransitionOutcome::Success(value) => value,
        outcome => panic!("value should validate: {outcome:?}"),
    }
}