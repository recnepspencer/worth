pub(crate) use {tempfile::tempdir};
pub(crate) use {worth_foundational::aspects, worth_foundational::AbsenceLaw, worth_foundational::AspectEquivalenceBasis, worth_foundational::AspectEvolutionPolicy, worth_foundational::AspectMask, worth_foundational::AspectValue, worth_foundational::ProjectionMask, worth_foundational::ScalarAspectType};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::aspect_native::StoreAspectAuthorityInput, worth_store::aspect_native::StoreAspectBoundaryFact, worth_store::aspect_native::StoreAspectContractAdmission, worth_store::aspect_native::StoreAspectIdentity, worth_store::aspect_native::StoreAspectNativeDenial, worth_store::aspect_native::StoreAspectPatchAuthorityInput, worth_store::aspect_native::StoreAspectPatchBoundaryFact};
pub(crate) use {worth_store::physical_runtime::PhysicalReadWorkRequest, worth_store::physical_runtime::PhysicalWorkProfileDeclaration, worth_store::physical_runtime::PhysicalWorkSemanticBasis, worth_store::physical_runtime::PhysicalWorkSemanticBasisDenial, worth_store::physical_runtime::PhysicalWorkSubmissionDenial};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_contract, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::alternative_physical_witness, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope_from_authority, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value};

pub(crate) fn projection_fact(
    contract: &worth_foundational::AspectContract,
    identity: worth_store::aspect_native::StoreAspectIdentity,
    witness: worth_store::aspect_native::StorePhysicalBoundaryWitness,
    value: &str,
) -> StoreAspectBoundaryFact {
    let state = match aspects()
        .authoritative_state()
        .admit([validated_value(contract, value)])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state should admit: {outcome:?}"),
    };
    worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        identity,
        worth_store::aspect_native::StoreAspectAuthorityInput::new(state, witness),
    )
    .unwrap()
}

pub(crate) fn request_scope() -> worth_store::physical_runtime::PhysicalWorkScope {
    worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            0,
            8,
        )
        .unwrap(),
    )
}