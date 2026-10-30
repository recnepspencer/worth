pub use worth_foundational::aspects;
pub use worth_proof::TransitionOutcome;
pub use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
pub use {
    worth_store::aspect_native::StoreAspectPatchAuthorityInput,
    worth_store::aspect_native::StoreAspectPatchBoundaryFact,
};
pub use {
    worth_store::physical_runtime::PhysicalMutationWorkRequest,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkScope,
    worth_store::physical_runtime::PhysicalWorkSemanticBasis,
};
pub use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub use {
    crate::physical_record_journeys::physical_work::fixture::authority::admitted_contract,
    crate::physical_record_journeys::physical_work::fixture::authority::security_scope,
    crate::physical_record_journeys::physical_work::fixture::authority::validated_value,
};

pub fn disjoint_mutation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    mutation_pair_fixture(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            16,
            8,
        )
        .unwrap(),
    )
}

pub fn foreground_saturation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    [PhysicalMutationWorkRequest; 4],
) {
    let (profile, basis, security) = mutation_basis();
    let request = |offset| {
        worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
            worth_store::physical_runtime::PhysicalWorkScope::one(
                worth_store_physical_format::RecordFrameCoordinate::new(
                    worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                    offset,
                    8,
                )
                .unwrap(),
            ),
            basis.clone(),
            security,
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap()
    };
    (profile, [request(8), request(16), request(24), request(32)])
}

pub fn overlapping_mutation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    mutation_pair_fixture(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            12,
            8,
        )
        .unwrap(),
    )
}

pub fn whole_catalog_mutation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    let (profile, basis, security) = mutation_basis();
    let range = worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
        worth_store::physical_runtime::PhysicalWorkScope::one(
            worth_store_physical_format::RecordFrameCoordinate::new(
                worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                8,
                8,
            )
            .unwrap(),
        ),
        basis.clone(),
        security,
        worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    )
    .unwrap();
    let whole = worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
        worth_store::physical_runtime::PhysicalWorkScope::artifact(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
        ),
        basis,
        security,
        worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    )
    .unwrap();
    (profile, range, whole)
}

pub fn disjoint_artifact_mutation_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    mutation_pair_fixture(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 },
            0,
            8,
        )
        .unwrap(),
    )
}

pub fn mutation_basis() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalWorkSemanticBasis,
    worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
) {
    let (contract, identity, contract_admission, physical_witness) = admitted_contract(1);
    let patch = match aspects()
        .patch()
        .whole_aspect()
        .set(validated_value(&contract, "disjoint-mutations"))
        .finish()
    {
        worth_proof::TransitionOutcome::Success(patch) => patch,
        outcome => panic!("patch should construct: {outcome:?}"),
    };
    let patch_fact =
        worth_store::aspect_native::StoreAspectPatchBoundaryFact::from_authoritative_patch(
            identity,
            worth_store::aspect_native::StoreAspectPatchAuthorityInput::new(
                patch,
                physical_witness,
            ),
        )
        .unwrap();
    let basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::mutation(
        patch_fact,
        contract_admission.clone(),
    )
    .unwrap();
    (
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(
            security_scope(physical_witness),
            [contract_admission],
        )
        .unwrap(),
        basis,
        security_scope(physical_witness),
    )
}

pub fn mutation_pair_fixture(
    second_coordinate: RecordFrameCoordinate,
) -> (
    PhysicalWorkProfileDeclaration,
    PhysicalMutationWorkRequest,
    PhysicalMutationWorkRequest,
) {
    let (profile, basis, security) = mutation_basis();
    let request = |offset, basis| {
        worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
            worth_store::physical_runtime::PhysicalWorkScope::one(
                worth_store_physical_format::RecordFrameCoordinate::new(
                    worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                    offset,
                    8,
                )
                .unwrap(),
            ),
            basis,
            security,
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap()
    };
    (
        profile,
        request(8, basis.clone()),
        worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
            worth_store::physical_runtime::PhysicalWorkScope::one(second_coordinate),
            basis,
            security,
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap(),
    )
}
