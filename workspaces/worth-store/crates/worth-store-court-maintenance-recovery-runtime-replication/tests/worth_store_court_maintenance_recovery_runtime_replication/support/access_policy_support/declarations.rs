pub(crate) use worth_store_certification::S6AccessPolicyEvidenceRow;
pub(crate) use worth_store_security::admitted_store_internal_security_scope_for_io_qos_test;
pub(crate) use {
    worth_store_physical_backend::AccessPolicyAdmission,
    worth_store_physical_backend::AccessPolicyBufferLifecycle,
    worth_store_physical_backend::AccessPolicyExecutionObservation,
    worth_store_physical_backend::AccessPolicyExecutionRequest,
    worth_store_physical_backend::AccessPolicyExecutionSession,
    worth_store_physical_backend::AccessPolicyRequest,
    worth_store_physical_backend::AccessPolicySecurityScope,
    worth_store_physical_backend::AdmittedAccessPolicy,
    worth_store_physical_backend::MixedAccessTransition,
    worth_store_physical_backend::PageCachePolicyProof,
    worth_store_physical_backend::PhysicalStoreAccessPolicyExecutor,
    worth_store_physical_backend::StoreAccessMode,
    worth_store_physical_backend::StoreAccessPolicyProofAuthority,
    worth_store_physical_backend::StoreOwnedAccessPolicyExecution,
};
pub(crate) use {
    worth_store_physical_format::PhysicalGeneration,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalRecordSlot,
    worth_store_physical_format::PhysicalReference,
    worth_store_physical_format::PhysicalReferenceAuthority,
    worth_store_physical_format::PhysicalSegmentId,
};

pub(crate) fn base_request(
    backend: &worth_store_physical_backend::AdmittedBackendCapabilityWitness,
    request: AccessPolicyRequest,
) -> AccessPolicyRequest {
    request
        .for_physical_reference(test_reference())
        .with_security_scope(test_security_scope())
        .with_buffer_lifecycle(pinned_lifecycle())
        .with_page_cache_policy(page_cache_policy(backend))
}

pub(crate) fn admit(
    backend: &worth_store_physical_backend::AdmittedBackendCapabilityWitness,
    request: AccessPolicyRequest,
) -> AdmittedAccessPolicy {
    worth_store_physical_backend::AccessPolicyAdmission::for_backend(backend)
        .admit(request)
        .expect("access policy admits")
}

pub(crate) fn executed_row(
    admitted: AdmittedAccessPolicy,
    expected: ExpectedExecutionRequest,
    observation: AccessPolicyExecutionObservation,
) -> S6AccessPolicyEvidenceRow {
    let mut executor =
        crate::access_policy_support::AssertingAccessBackend::new(expected, observation);
    let receipt = worth_store_physical_backend::AccessPolicyExecutionSession::for_store_backend(
        &mut executor,
        worth_store_physical_backend::StoreOwnedAccessPolicyExecution::for_certification_test_authority(),
    )
    .execute(admitted)
    .expect("backend executes")
    .expect("execution succeeds");
    worth_store_certification::S6AccessPolicyEvidenceRow::from_execution_receipt(receipt)
}

pub(crate) fn violation_row(
    admitted: AdmittedAccessPolicy,
    expected: ExpectedExecutionRequest,
    observation: AccessPolicyExecutionObservation,
) -> S6AccessPolicyEvidenceRow {
    let mode = admitted.mode();
    let mut executor =
        crate::access_policy_support::AssertingAccessBackend::new(expected, observation);
    let violation = worth_store_physical_backend::AccessPolicyExecutionSession::for_store_backend(
        &mut executor,
        worth_store_physical_backend::StoreOwnedAccessPolicyExecution::for_certification_test_authority(),
    )
    .execute(admitted)
    .expect("backend executes")
    .expect_err("execution violates");
    worth_store_certification::S6AccessPolicyEvidenceRow::from_violation(mode, violation)
}

pub(crate) fn mmap_request(
    backend: &worth_store_physical_backend::AdmittedBackendCapabilityWitness,
) -> AccessPolicyRequest {
    base_request(
        backend,
        worth_store_physical_backend::AccessPolicyRequest::mmap_read()
            .with_mmap_fault_posture(admitted_mmap_posture(backend)),
    )
}

pub(crate) fn pinned_lifecycle() -> AccessPolicyBufferLifecycle {
    worth_store_physical_backend::AccessPolicyBufferLifecycle::for_certification_pinned_physical_substrate_lease()
}

pub(crate) fn test_reference() -> PhysicalReference {
    let cell =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
            .slot_cell(
                worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap(),
                worth_store_physical_format::PhysicalPageId::from_raw(1).unwrap(),
                worth_store_physical_format::PhysicalRecordSlot::from_raw(1).unwrap(),
            )
            .with_slot_generation(
                worth_store_physical_format::PhysicalGeneration::from_raw(1).unwrap(),
            );
    worth_store_physical_format::PhysicalReferenceAuthority::for_canonical_physical_format()
        .admit_page_slot(cell)
        .reference()
}

pub(crate) fn page_cache_policy(
    backend: &worth_store_physical_backend::AdmittedBackendCapabilityWitness,
) -> PageCachePolicyProof {
    worth_store_physical_backend::StoreAccessPolicyProofAuthority::for_admitted_backend(backend)
        .page_cache_policy()
        .expect("test backend carries Store-admitted page-cache posture")
}

pub(crate) fn admitted_mmap_posture(
    backend: &worth_store_physical_backend::AdmittedBackendCapabilityWitness,
) -> worth_store_physical_backend::MmapFaultPosture {
    worth_store_physical_backend::StoreAccessPolicyProofAuthority::for_admitted_backend(backend)
        .mmap_posture()
        .expect("test backend carries mmap capability posture")
}

pub(crate) fn test_security_scope() -> AccessPolicySecurityScope {
    let admitted = admitted_store_internal_security_scope_for_io_qos_test();
    worth_store_physical_backend::AccessPolicySecurityScope::from_current_store_scope(
        admitted.witnesses(),
    )
}

#[derive(Clone, Copy)]
pub(crate) struct ExpectedExecutionRequest {
    pub(crate) mode: StoreAccessMode,
    pub(crate) reference: PhysicalReference,
    pub(crate) transition: Option<MixedAccessTransition>,
    pub(crate) requires_direct_io: bool,
    pub(crate) requires_mmap: bool,
}

impl ExpectedExecutionRequest {
    pub(crate) fn buffered() -> Self {
        Self::new(
            worth_store_physical_backend::StoreAccessMode::Buffered,
            None,
            false,
            false,
        )
    }

    pub(crate) fn mmap() -> Self {
        Self::new(
            worth_store_physical_backend::StoreAccessMode::Mmap,
            None,
            false,
            true,
        )
    }

    pub(crate) fn new(
        mode: StoreAccessMode,
        transition: Option<MixedAccessTransition>,
        requires_direct_io: bool,
        requires_mmap: bool,
    ) -> Self {
        Self {
            mode,
            reference: test_reference(),
            transition,
            requires_direct_io,
            requires_mmap,
        }
    }
}

pub(crate) struct AssertingAccessBackend {
    pub(crate) expected: ExpectedExecutionRequest,
    pub(crate) observation: AccessPolicyExecutionObservation,
}

impl AssertingAccessBackend {
    pub(crate) const fn new(
        expected: ExpectedExecutionRequest,
        observation: AccessPolicyExecutionObservation,
    ) -> Self {
        Self {
            expected,
            observation,
        }
    }
}

impl PhysicalStoreAccessPolicyExecutor for AssertingAccessBackend {
    type Error = core::convert::Infallible;

    fn execute_access_policy(
        &mut self,
        request: AccessPolicyExecutionRequest,
    ) -> Result<AccessPolicyExecutionObservation, Self::Error> {
        let access = request.access_request();
        assert_eq!(access.mode(), self.expected.mode);
        assert_eq!(access.reference(), Some(self.expected.reference));
        assert!(access.security_scope().is_some());
        assert!(access.page_cache_policy().is_some());
        assert_eq!(access.mixed_transition(), self.expected.transition);
        assert_eq!(
            access.alignment().is_some(),
            self.expected.requires_direct_io
        );
        assert!(!self.expected.requires_mmap || access.mmap_fault_posture().admits_mmap());
        if self.expected.transition.is_some() {
            assert!(access.coherence_basis().is_some());
        }
        Ok(self.observation)
    }
}
