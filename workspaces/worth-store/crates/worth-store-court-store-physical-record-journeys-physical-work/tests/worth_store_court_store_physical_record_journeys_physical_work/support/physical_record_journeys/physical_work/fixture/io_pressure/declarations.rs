pub(crate) use {worth_foundational::aspects};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::aspect_native::StoreAspectAuthorityInput, worth_store::aspect_native::StoreAspectBoundaryFact, worth_store::aspect_native::StoreAspectPatchAuthorityInput, worth_store::aspect_native::StoreAspectPatchBoundaryFact};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationWorkRequest, worth_store::physical_runtime::PhysicalReadWorkRequest, worth_store::physical_runtime::PhysicalWorkProfileDeclaration, worth_store::physical_runtime::PhysicalWorkScope, worth_store::physical_runtime::PhysicalWorkSemanticBasis};
pub(crate) use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement};
pub(crate) use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

pub(crate) fn disjoint_io_pressure_fixture() -> (
    PhysicalWorkProfileDeclaration,
    [PhysicalReadWorkRequest; 2],
    [PhysicalMutationWorkRequest; 2],
) {
    let (contract, identity, contract_admission, witness) = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_contract(1);
    let security = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope(witness);
    let profile =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security, [contract_admission.clone()]).unwrap();
    let read_state = match aspects()
        .authoritative_state()
        .admit([worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value(&contract, "pressure-read")])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("pressure read state should admit: {outcome:?}"),
    };
    let read_fact = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        identity.clone(),
        worth_store::aspect_native::StoreAspectAuthorityInput::new(read_state, witness),
    )
    .unwrap();
    let read_basis =
        worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(read_fact, contract_admission.clone()).unwrap();
    let patch = match aspects()
        .patch()
        .whole_aspect()
        .set(worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value(&contract, "pressure-write"))
        .finish()
    {
        worth_proof::TransitionOutcome::Success(patch) => patch,
        outcome => panic!("pressure mutation patch should admit: {outcome:?}"),
    };
    let patch_fact = worth_store::aspect_native::StoreAspectPatchBoundaryFact::from_authoritative_patch(
        identity,
        worth_store::aspect_native::StoreAspectPatchAuthorityInput::new(patch, witness),
    )
    .unwrap();
    let write_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::mutation(patch_fact, contract_admission).unwrap();
    let bootstrap_read =
        worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, 0, 8).unwrap();
    let root_read =
        worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 }, 0, 8)
            .unwrap();
    let bootstrap_write =
        worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap();
    let root_write =
        worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 }, 8, 8)
            .unwrap();
    (
        profile,
        [
            read_request(bootstrap_read, read_basis.clone(), security),
            read_request(root_read, read_basis, security),
        ],
        [
            write_request(bootstrap_write, write_basis.clone(), security),
            write_request(root_write, write_basis, security),
        ],
    )
}

pub(crate) fn read_request(
    coordinate: RecordFrameCoordinate,
    basis: PhysicalWorkSemanticBasis,
    security: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
) -> PhysicalReadWorkRequest {
    worth_store::physical_runtime::PhysicalReadWorkRequest::new(worth_store::physical_runtime::PhysicalWorkScope::one(coordinate), basis, security).unwrap()
}

pub(crate) fn write_request(
    coordinate: RecordFrameCoordinate,
    basis: PhysicalWorkSemanticBasis,
    security: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
) -> PhysicalMutationWorkRequest {
    worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
        worth_store::physical_runtime::PhysicalWorkScope::one(coordinate),
        basis,
        security,
        worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    )
    .unwrap()
}