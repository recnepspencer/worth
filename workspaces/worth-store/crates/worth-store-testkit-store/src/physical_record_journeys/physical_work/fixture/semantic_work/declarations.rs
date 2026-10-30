pub use worth_foundational::aspects;
pub use worth_proof::TransitionOutcome;
pub use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
pub use {
    worth_store::aspect_native::StoreAspectAuthorityInput,
    worth_store::aspect_native::StoreAspectBoundaryFact,
    worth_store::aspect_native::StoreAspectPatchAuthorityInput,
    worth_store::aspect_native::StoreAspectPatchBoundaryFact,
};
pub use {
    worth_store::physical_runtime::PhysicalMutationWorkRequest,
    worth_store::physical_runtime::PhysicalReadWorkRequest,
    worth_store::physical_runtime::PhysicalSignalAspectBindingSet,
    worth_store::physical_runtime::PhysicalSignalAspectDeclaration,
    worth_store::physical_runtime::PhysicalSignalAspectRole,
    worth_store::physical_runtime::PhysicalWorkAspectDelta,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkScope,
    worth_store::physical_runtime::PhysicalWorkSemanticBasis,
    worth_store::physical_runtime::PhysicalWorkSignalFamily,
    worth_store::physical_runtime::PhysicalWorkSignalFamilySet,
};
pub use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub use {
    crate::physical_record_journeys::physical_work::fixture::authority::admitted_contract,
    crate::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract,
    crate::physical_record_journeys::physical_work::fixture::authority::security_scope,
    crate::physical_record_journeys::physical_work::fixture::authority::validated_value,
};

pub const EXPECTED_NATIVE_RECORD_BINDING_COUNT: u16 = 12;

pub fn work_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalReadWorkRequest,
    PhysicalMutationWorkRequest,
) {
    let (contract, identity, contract_admission, physical_witness) = admitted_contract(1);
    let security = security_scope(physical_witness);
    let profile = worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(
        security,
        [contract_admission.clone()],
    )
    .unwrap();
    let read_state = match aspects()
        .authoritative_state()
        .admit([validated_value(&contract, "read")])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state should admit: {outcome:?}"),
    };
    let read_fact = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        identity.clone(),
        worth_store::aspect_native::StoreAspectAuthorityInput::new(read_state, physical_witness),
    )
    .unwrap();
    let read_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        read_fact,
        contract_admission.clone(),
    )
    .unwrap();
    let patch = match aspects()
        .patch()
        .whole_aspect()
        .set(validated_value(&contract, "mutation"))
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
    let mutation_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::mutation(
        patch_fact,
        contract_admission,
    )
    .unwrap();
    let read_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            0,
            8,
        )
        .unwrap(),
    );
    let mutation_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            8,
            8,
        )
        .unwrap(),
    );
    (
        profile,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            read_scope, read_basis, security,
        )
        .unwrap(),
        worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
            mutation_scope,
            mutation_basis,
            security,
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap(),
    )
}

pub fn matching_aspect_delta(revision: u64, value: &str) -> PhysicalWorkAspectDelta {
    let (contract, identity, admission, witness) = admitted_contract(revision);
    let bindings = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(
            security_scope(witness),
            [admission],
        )
        .unwrap(),
    );
    let state = match aspects()
        .authoritative_state()
        .admit([validated_value(&contract, value)])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state should admit: {outcome:?}"),
    };
    let fact = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        identity,
        worth_store::aspect_native::StoreAspectAuthorityInput::new(state, witness),
    )
    .unwrap();
    worth_store::physical_runtime::PhysicalWorkAspectDelta::from_boundary_fact(
        bindings.binding_for_slot(0).unwrap(),
        &fact,
        worth_store::physical_runtime::PhysicalWorkScope::one(
            worth_store_physical_format::RecordFrameCoordinate::new(
                worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                0,
                8,
            )
            .unwrap(),
        ),
    )
    .unwrap()
}

pub fn family_locality_fixture() -> (
    PhysicalWorkProfileDeclaration,
    PhysicalReadWorkRequest,
    PhysicalMutationWorkRequest,
    PhysicalWorkAspectDelta,
) {
    let (read_contract, read_identity, read_admission, witness) =
        admitted_named_contract("store.physical.work.read-availability", 81, 1);
    let (write_contract, write_identity, write_admission, _) =
        admitted_named_contract("store.physical.work.write-eligibility", 82, 1);
    let profile =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
            security_scope(witness),
            [
                worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
                    read_admission.clone(),
                    worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency,
                )
                .for_families(
                    worth_store::physical_runtime::PhysicalWorkSignalFamilySet::only(
                        worth_store::physical_runtime::PhysicalWorkSignalFamily::ReadFault,
                    ),
                ),
                worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
                    write_admission.clone(),
                    worth_store::physical_runtime::PhysicalSignalAspectRole::DependencyAndOutput,
                )
                .for_families(
                    worth_store::physical_runtime::PhysicalWorkSignalFamilySet::only(
                        worth_store::physical_runtime::PhysicalWorkSignalFamily::ExactWriteback,
                    ),
                ),
            ],
        )
        .unwrap();
    let read_state = match aspects()
        .authoritative_state()
        .admit([validated_value(&read_contract, "available")])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state should admit: {outcome:?}"),
    };
    let read_fact = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        read_identity.clone(),
        worth_store::aspect_native::StoreAspectAuthorityInput::new(read_state, witness),
    )
    .unwrap();
    let read_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        read_fact.clone(),
        read_admission,
    )
    .unwrap();
    let write_patch = match aspects()
        .patch()
        .whole_aspect()
        .set(validated_value(&write_contract, "eligible"))
        .finish()
    {
        worth_proof::TransitionOutcome::Success(patch) => patch,
        outcome => panic!("patch should construct: {outcome:?}"),
    };
    let write_fact =
        worth_store::aspect_native::StoreAspectPatchBoundaryFact::from_authoritative_patch(
            write_identity,
            worth_store::aspect_native::StoreAspectPatchAuthorityInput::new(write_patch, witness),
        )
        .unwrap();
    let write_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::mutation(
        write_fact,
        write_admission,
    )
    .unwrap();
    let bindings = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
        profile.clone(),
    );
    let read_binding = bindings.binding_for_identity(&read_identity).unwrap();
    let read_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            0,
            8,
        )
        .unwrap(),
    );
    let delta = worth_store::physical_runtime::PhysicalWorkAspectDelta::from_boundary_fact(
        read_binding,
        &read_fact,
        read_scope.clone(),
    )
    .unwrap();
    let security = security_scope(witness);
    let read = worth_store::physical_runtime::PhysicalReadWorkRequest::new(
        read_scope, read_basis, security,
    )
    .unwrap();
    let write = worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
        worth_store::physical_runtime::PhysicalWorkScope::one(
            worth_store_physical_format::RecordFrameCoordinate::new(
                worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                8,
                8,
            )
            .unwrap(),
        ),
        write_basis,
        security,
        worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    )
    .unwrap();
    (profile, read, write, delta)
}
