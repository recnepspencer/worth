pub(crate) use worth_store_physical_format::PhysicalAlignmentClass;
pub(crate) use {
    crate::access_policy_support::base_request, crate::access_policy_support::page_cache_policy,
    crate::access_policy_support::pinned_lifecycle, crate::access_policy_support::test_reference,
    crate::access_policy_support::test_security_scope,
    crate::access_policy_support::ExpectedExecutionRequest,
};
pub(crate) use {
    worth_store_physical_backend::AccessPolicyRequest,
    worth_store_physical_backend::BackendCapabilityAdmissionRequest,
    worth_store_physical_backend::BackendCapabilityEvidenceBasis,
    worth_store_physical_backend::BackendCapabilityKind,
    worth_store_physical_backend::BackendCapabilitySupportPosture,
    worth_store_physical_backend::BackendCapabilitySupportSet,
    worth_store_physical_backend::BackendMediaAssumptionSet,
    worth_store_physical_backend::BackendRebindTriggers,
    worth_store_physical_backend::BackendTargetProfile,
    worth_store_physical_backend::MixedAccessTransition,
    worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority,
    worth_store_physical_backend::StoreAccessMode,
    worth_store_physical_backend::StoreAccessPolicyProofAuthority,
};

pub(crate) fn direct_io_request(
    backend: &worth_store_physical_backend::AdmittedBackendCapabilityWitness,
) -> AccessPolicyRequest {
    base_request(
        backend,
        worth_store_physical_backend::AccessPolicyRequest::direct_io_read()
            .with_alignment_requirement(
            worth_store_physical_backend::StoreAccessPolicyProofAuthority::for_admitted_backend(
                backend,
            )
            .direct_io_page_and_sector_aligned(
                test_reference(),
                pinned_lifecycle(),
                4096,
                worth_store_physical_format::PhysicalAlignmentClass::page_start_4k(),
                worth_store_physical_format::PhysicalAlignmentClass::extent_start_4k(),
            )
            .expect("test backend carries direct I/O alignment assumptions"),
        ),
    )
}

pub(crate) fn mixed_request(
    backend: &worth_store_physical_backend::AdmittedBackendCapabilityWitness,
    transition: MixedAccessTransition,
) -> AccessPolicyRequest {
    let reference = test_reference();
    let scope = test_security_scope();
    worth_store_physical_backend::AccessPolicyRequest::mixed_read(transition)
        .for_physical_reference(reference)
        .with_security_scope(scope)
        .with_buffer_lifecycle(pinned_lifecycle())
        .with_page_cache_policy(page_cache_policy(backend))
        .with_alignment_requirement(
            worth_store_physical_backend::StoreAccessPolicyProofAuthority::for_admitted_backend(
                backend,
            )
            .direct_io_page_and_sector_aligned(
                reference,
                pinned_lifecycle(),
                4096,
                worth_store_physical_format::PhysicalAlignmentClass::page_start_4k(),
                worth_store_physical_format::PhysicalAlignmentClass::extent_start_4k(),
            )
            .expect("test backend carries direct I/O alignment assumptions"),
        )
        .with_coherence_basis(
            worth_store_physical_backend::StoreAccessPolicyProofAuthority::for_admitted_backend(
                backend,
            )
            .mixed_coherence(transition, reference, scope)
            .expect("test backend carries mixed access coherence assumptions"),
        )
}

pub(crate) fn backend_with_access(
    direct_io: BackendCapabilitySupportPosture,
) -> worth_store_physical_backend::AdmittedBackendCapabilityWitness {
    let support = worth_store_physical_backend::BackendCapabilitySupportSet::all_supported()
        .with_posture(
            worth_store_physical_backend::BackendCapabilityKind::DirectIo,
            direct_io,
        );
    let assumptions =
        worth_store_physical_backend::BackendMediaAssumptionSet::platform_file_defaults()
            .with_direct_io_alignment()
            .with_sector_atomicity()
            .with_page_cache_policy()
            .with_mmap_coherence()
            .with_mixed_access_coherence();
    worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority::store_owned()
        .admit_backend_capability(worth_store_physical_backend::BackendCapabilityAdmissionRequest::new(
            worth_store_physical_backend::BackendTargetProfile::PosixFileFsyncDirSync,
            worth_store_physical_backend::BackendCapabilityEvidenceBasis::certified_backend_profile(),
            support,
            assumptions,
            worth_store_physical_backend::BackendRebindTriggers::kernel_filesystem_mount_firmware_and_backend()
                .with_sector_alignment()
                .with_security_posture(),
        ))
        .expect("backend admits")
}

impl ExpectedExecutionRequest {
    pub(crate) fn direct_io() -> Self {
        Self::new(
            worth_store_physical_backend::StoreAccessMode::DirectIo,
            None,
            true,
            false,
        )
    }

    pub(crate) fn mixed(transition: MixedAccessTransition) -> Self {
        Self::new(
            worth_store_physical_backend::StoreAccessMode::Mixed,
            Some(transition),
            true,
            false,
        )
    }
}
