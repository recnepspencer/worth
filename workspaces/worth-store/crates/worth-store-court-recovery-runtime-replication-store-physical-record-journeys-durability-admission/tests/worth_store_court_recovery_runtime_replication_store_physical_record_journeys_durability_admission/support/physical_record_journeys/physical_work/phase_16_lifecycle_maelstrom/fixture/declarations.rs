pub(crate) use std::path::Path;

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_signal::facade::PartitionSubscription;
pub(crate) use worth_store_security::StoreAuthorityBoundSecurityScopeReceipt;
pub(crate) use {worth_foundational::aspects, worth_foundational::AspectContract};
pub(crate) use {
    worth_store::aspect_native::StoreAspectAuthorityInput,
    worth_store::aspect_native::StoreAspectBoundaryFact,
    worth_store::aspect_native::StoreAspectContractAdmission,
    worth_store::aspect_native::StoreAspectIdentity,
    worth_store::aspect_native::StoreAspectPatchAuthorityInput,
    worth_store::aspect_native::StoreAspectPatchBoundaryFact,
    worth_store::aspect_native::StorePhysicalBoundaryWitness,
};
pub(crate) use {
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalMutationWorkRequest,
    worth_store::physical_runtime::PhysicalReadWorkRequest,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalSignalAspectBindingSet,
    worth_store::physical_runtime::PhysicalSignalAspectDeclaration,
    worth_store::physical_runtime::PhysicalSignalAspectRole,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::PhysicalWorkAspectDelta,
    worth_store::physical_runtime::PhysicalWorkCapacity,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkScope,
    worth_store::physical_runtime::PhysicalWorkSemanticBasis,
    worth_store::physical_runtime::PhysicalWorkSignalFamily,
    worth_store::physical_runtime::PhysicalWorkSignalFamilySet,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement,
    worth_store_physical_backend::CertificationMediaFaultActivation,
    worth_store_physical_backend::CertificationMediaFaultAuthority,
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaFaultDirective,
    worth_store_physical_backend::MediaFaultRule, worth_store_physical_backend::MediaOperationRole,
    worth_store_physical_backend::MediaPauseGate,
};
pub(crate) use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub(crate) struct MaelstromFixture {
    pub(crate) profile: PhysicalWorkProfileDeclaration,
    pub(crate) reads: [PhysicalReadWorkRequest; 2],
    pub(crate) write: PhysicalMutationWorkRequest,
    pub(crate) read_delta: PhysicalWorkAspectDelta,
}

pub(crate) struct MaelstromPauseGates {
    pub(crate) first_read: MediaPauseGate,
    pub(crate) second_read: MediaPauseGate,
    pub(crate) close_read: MediaPauseGate,
    pub(crate) close_read_activation: CertificationMediaFaultActivation,
}

pub(crate) struct AdmittedBinding {
    pub(crate) contract: AspectContract,
    pub(crate) identity: StoreAspectIdentity,
    pub(crate) admission: StoreAspectContractAdmission,
}

pub(crate) struct MaelstromBindings {
    pub(crate) first_read: AdmittedBinding,
    pub(crate) second_read: AdmittedBinding,
    pub(crate) write: AdmittedBinding,
    pub(crate) witness: StorePhysicalBoundaryWitness,
    pub(crate) security: StoreAuthorityBoundSecurityScopeReceipt,
}

pub(crate) struct MaelstromSemanticBases {
    pub(crate) first_read: PhysicalWorkSemanticBasis,
    pub(crate) second_read: PhysicalWorkSemanticBasis,
    pub(crate) write: PhysicalWorkSemanticBasis,
    pub(crate) first_read_fact: StoreAspectBoundaryFact,
}

pub(crate) fn maelstrom_fixture() -> MaelstromFixture {
    let admitted = admitted_bindings();
    let profile = physical_work_profile(&admitted);
    let bases = semantic_bases(&admitted);
    let first_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            0,
            8,
        )
        .unwrap(),
    );
    let second_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 },
            0,
            8,
        )
        .unwrap(),
    );
    let profile_bindings =
        worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
            profile.clone(),
        );
    let read_delta = worth_store::physical_runtime::PhysicalWorkAspectDelta::from_boundary_fact(
        profile_bindings
            .binding_for_identity(&admitted.first_read.identity)
            .unwrap(),
        &bases.first_read_fact,
        first_scope.clone(),
    )
    .unwrap();
    let reads = [
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            first_scope,
            bases.first_read,
            admitted.security,
        )
        .unwrap(),
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            second_scope,
            bases.second_read,
            admitted.security,
        )
        .unwrap(),
    ];
    let write = worth_store::physical_runtime::PhysicalMutationWorkRequest::exact_write(
        worth_store::physical_runtime::PhysicalWorkScope::one(
            worth_store_physical_format::RecordFrameCoordinate::new(
                worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                8,
                8,
            )
            .unwrap(),
        ),
        bases.write,
        admitted.security,
        worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    )
    .unwrap();
    MaelstromFixture {
        profile,
        reads,
        write,
        read_delta,
    }
}

pub(crate) fn admitted_bindings() -> MaelstromBindings {
    let (contract, identity, admission, witness) = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract(
        "store.physical.work.phase-16-read-left",
        961,
        1,
    );
    let first_read = AdmittedBinding {
        contract,
        identity,
        admission,
    };
    let (contract, identity, admission, _) = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract(
        "store.physical.work.phase-16-read-right",
        962,
        1,
    );
    let second_read = AdmittedBinding {
        contract,
        identity,
        admission,
    };
    let (contract, identity, admission, _) = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract(
        "store.physical.work.phase-16-write",
        963,
        1,
    );
    MaelstromBindings {
        first_read,
        second_read,
        write: AdmittedBinding {
            contract,
            identity,
            admission,
        },
        witness,
        security: worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope(witness),
    }
}

pub(crate) fn physical_work_profile(
    bindings: &MaelstromBindings,
) -> PhysicalWorkProfileDeclaration {
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
        bindings.security,
        [
            read_aspect(&bindings.first_read, "bootstrap-catalog", "frame-0"),
            read_aspect(
                &bindings.second_read,
                "root-manifest",
                "generation-1-frame-0",
            ),
            worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
                bindings.write.admission.clone(),
                worth_store::physical_runtime::PhysicalSignalAspectRole::DependencyAndOutput,
            )
            .with_partition(
                worth_signal::facade::PartitionSubscription::partition_and_detail(
                    "bootstrap-catalog",
                    "frame-0",
                ),
            ),
        ],
    )
    .unwrap()
    .with_capacity(
        worth_store::physical_runtime::PhysicalWorkCapacity::new(32, 4, 32, 16_384, 32_768)
            .unwrap()
            .with_terminal_evidence_capacity(64)
            .unwrap(),
    )
}

pub(crate) fn read_aspect(
    binding: &AdmittedBinding,
    partition: &str,
    detail: &str,
) -> PhysicalSignalAspectDeclaration {
    worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
        binding.admission.clone(),
        worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency,
    )
    .with_partition(
        worth_signal::facade::PartitionSubscription::partition_and_detail(partition, detail),
    )
    .for_families(
        worth_store::physical_runtime::PhysicalWorkSignalFamilySet::only(
            worth_store::physical_runtime::PhysicalWorkSignalFamily::ReadFault,
        ),
    )
}

pub(crate) fn semantic_bases(bindings: &MaelstromBindings) -> MaelstromSemanticBases {
    let (first_read, first_read_fact) =
        read_basis(&bindings.first_read, bindings.witness, "available");
    let (second_read, _) = read_basis(&bindings.second_read, bindings.witness, "available");
    let write = write_basis(&bindings.write, bindings.witness);
    MaelstromSemanticBases {
        first_read,
        second_read,
        write,
        first_read_fact,
    }
}

pub(crate) fn read_basis(
    binding: &AdmittedBinding,
    witness: StorePhysicalBoundaryWitness,
    value: &str,
) -> (PhysicalWorkSemanticBasis, StoreAspectBoundaryFact) {
    let state =
        match aspects()
            .authoritative_state()
            .admit([worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value(
                &binding.contract,
                value,
            )]) {
            worth_proof::TransitionOutcome::Success(state) => state,
            outcome => panic!("Phase 16 read state should admit: {outcome:?}"),
        };
    let fact = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        binding.identity.clone(),
        worth_store::aspect_native::StoreAspectAuthorityInput::new(state, witness),
    )
    .unwrap();
    let basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        fact.clone(),
        binding.admission.clone(),
    )
    .unwrap();
    (basis, fact)
}

pub(crate) fn write_basis(
    binding: &AdmittedBinding,
    witness: StorePhysicalBoundaryWitness,
) -> PhysicalWorkSemanticBasis {
    let patch = match aspects()
        .patch()
        .whole_aspect()
        .set(worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value(
            &binding.contract,
            "eligible",
        ))
        .finish()
    {
        worth_proof::TransitionOutcome::Success(patch) => patch,
        outcome => panic!("Phase 16 write patch should admit: {outcome:?}"),
    };
    let fact = worth_store::aspect_native::StoreAspectPatchBoundaryFact::from_authoritative_patch(
        binding.identity.clone(),
        worth_store::aspect_native::StoreAspectPatchAuthorityInput::new(patch, witness),
    )
    .unwrap();
    worth_store::physical_runtime::PhysicalWorkSemanticBasis::mutation(
        fact,
        binding.admission.clone(),
    )
    .unwrap()
}
