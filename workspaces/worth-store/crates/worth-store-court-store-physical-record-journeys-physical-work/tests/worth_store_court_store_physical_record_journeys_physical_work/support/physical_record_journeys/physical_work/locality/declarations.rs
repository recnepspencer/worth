pub(crate) use {tempfile::tempdir};
pub(crate) use {worth_foundational::aspects};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::aspect_native::StoreAspectAuthorityInput, worth_store::aspect_native::StoreAspectBoundaryFact};
pub(crate) use {worth_store::physical_runtime::PhysicalReadWorkRequest, worth_store::physical_runtime::PhysicalSignalAspectBindingSet, worth_store::physical_runtime::PhysicalSignalAspectDeclaration, worth_store::physical_runtime::PhysicalSignalAspectRole, worth_store::physical_runtime::PhysicalWorkAspectDelta, worth_store::physical_runtime::PhysicalWorkProfileDeclaration, worth_store::physical_runtime::PhysicalWorkReadiness, worth_store::physical_runtime::PhysicalWorkScope, worth_store::physical_runtime::PhysicalWorkSemanticBasis, worth_store::physical_runtime::PhysicalWorkSignalFamily, worth_store::physical_runtime::PhysicalWorkSignalFamilySet};
pub(crate) use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::EXPECTED_NATIVE_RECORD_BINDING_COUNT};

pub(crate) fn ready(readiness: PhysicalWorkReadiness) -> worth_store::physical_runtime::ReadyPhysicalWork {
    match readiness {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(_) => panic!("evaluated physical dependency must be ready"),
    }
}

pub(crate) fn read_dependency(
    admission: worth_store::aspect_native::StoreAspectContractAdmission,
) -> PhysicalSignalAspectDeclaration {
    worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(admission, worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency)
        .for_families(worth_store::physical_runtime::PhysicalWorkSignalFamilySet::only(
            worth_store::physical_runtime::PhysicalWorkSignalFamily::ReadFault,
        ))
}

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

pub(crate) fn request(
    basis: PhysicalWorkSemanticBasis,
    witness: worth_store::aspect_native::StorePhysicalBoundaryWitness,
    artifact: RecordArtifactFile,
    offset: u64,
) -> PhysicalReadWorkRequest {
    worth_store::physical_runtime::PhysicalReadWorkRequest::new(
        worth_store::physical_runtime::PhysicalWorkScope::one(worth_store_physical_format::RecordFrameCoordinate::new(artifact, offset, 8).unwrap()),
        basis,
        security_scope(witness),
    )
    .unwrap()
}

pub(crate) fn admit_read(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: PhysicalReadWorkRequest,
) -> worth_store::physical_runtime::AdmittedPhysicalWork {
    let receipt = match serving
        .physical_read_submission()
        .submit(request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("read should submit: {outcome:?}"),
    };
    serving.admit_physical_work(receipt).unwrap()
}