pub(crate) use tempfile::tempdir;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::PartitionSubscription;
pub(crate) use {
    worth_foundational::aspects, worth_foundational::AbsenceLaw,
    worth_foundational::AspectEquivalenceBasis, worth_foundational::AspectEvolutionPolicy,
    worth_foundational::AspectMask, worth_foundational::AspectValue,
    worth_foundational::ProjectionMask, worth_foundational::ScalarAspectType,
};
pub(crate) use {
    worth_store::aspect_native::StoreAspectAuthorityInput,
    worth_store::aspect_native::StoreAspectBoundaryFact,
    worth_store::aspect_native::StoreAspectContractAdmission,
    worth_store::aspect_native::StoreAspectPatchAuthorityInput,
    worth_store::aspect_native::StoreAspectPatchBoundaryFact,
};
pub(crate) use {
    worth_store::physical_runtime::PhysicalSignalAspectBindingSet,
    worth_store::physical_runtime::PhysicalSignalAspectDeclaration,
    worth_store::physical_runtime::PhysicalSignalAspectRole,
    worth_store::physical_runtime::PhysicalSignalDeltaApplicationFailure,
    worth_store::physical_runtime::PhysicalWorkAspectDelta,
    worth_store::physical_runtime::PhysicalWorkAspectDeltaDenial,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkScope,
};
pub(crate) use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_contract,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
};

pub(crate) fn delta_scope() -> PhysicalWorkScope {
    worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            0,
            8,
        )
        .unwrap(),
    )
}

pub(crate) fn boundary_fact(
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
