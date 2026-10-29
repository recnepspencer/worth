pub(crate) use {worth_foundational::performance, worth_foundational::FoundationalPerformanceAccessPatternPosture, worth_foundational::FoundationalPerformanceBoundary, worth_foundational::FoundationalPerformanceBreadthLocalityPosture, worth_foundational::FoundationalPerformanceBudgetKind, worth_foundational::FoundationalPerformanceEvidenceStrength, worth_foundational::FoundationalPerformanceExecutionTemperature, worth_foundational::FoundationalPerformanceFallbackDebtPosture, worth_foundational::FoundationalPerformanceFreshnessRetentionPosture, worth_foundational::FoundationalPerformanceWorkClass, worth_foundational::FoundationalPolicyAdmissionReceipt};
pub(crate) use {worth_store_contracts::QueueProducerResourceShape};
pub(crate) use {worth_store_io_scheduler::foreground_reservation::admitted_secure_frame_read_reservation_for_certification_test};
pub(crate) use {worth_store_io_scheduler::queue_execution::QueueExecutionViolationCause};
pub(crate) use {worth_store_io_scheduler::admit_queue_execution_plan, worth_store_io_scheduler::admit_secure_frame_backend_capability_for_scheduler_claim, worth_store_io_scheduler::admit_secure_io_scope_for_scheduler, worth_store_io_scheduler::admit_security_scope_for_scheduler, worth_store_io_scheduler::execute_ready_queue_plan, worth_store_io_scheduler::BackgroundResourceBudget, worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission, worth_store_io_scheduler::QueueExecutionAdmissionRequest, worth_store_io_scheduler::QueueExecutionOutcome, worth_store_io_scheduler::QueueExecutionReadyPlan, worth_store_io_scheduler::SecureIoOperation, worth_store_io_scheduler::SecureIoPostureRequirement, worth_store_io_scheduler::SecureIoPreservationRequest};
pub(crate) use {worth_store_physical_backend::AdmittedBackendCapabilityWitness, worth_store_physical_backend::BackendCapabilityAdmissionRequest, worth_store_physical_backend::BackendCapabilityEvidenceBasis, worth_store_physical_backend::BackendCapabilitySupportSet, worth_store_physical_backend::BackendMediaAssumptionSet, worth_store_physical_backend::BackendQueueExecutionAdaptation, worth_store_physical_backend::BackendQueueExecutionCompletion, worth_store_physical_backend::BackendQueueExecutionPosture, worth_store_physical_backend::BackendQueueSpeculativeScope, worth_store_physical_backend::BackendRebindTriggers, worth_store_physical_backend::BackendTargetProfile, worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority};
pub(crate) fn secure_frame_read_ahead_plan() -> QueueExecutionReadyPlan {
    let reservation = admitted_secure_frame_read_reservation_for_certification_test();
    let work = worth_store_test_support::harness::scheduling::scheduler_foreground_read_work(
        reservation,
        7,
        worth_store_contracts::QueueProducerResourceShape::new()
            .with_queue_slots(1)
            .with_read_ahead_windows(1)
            .with_worker_permits(1),
    )
    .expect("secure-frame foreground read should lower through scheduler");
    let security = io_qos_security_scope_admission();
    let (backend, _) = secure_frame_backend(&security);
    let secure_io = admit_secure_io_scope_for_scheduler(
        worth_store_io_scheduler::SecureIoPreservationRequest::new(worth_store_io_scheduler::SecureIoOperation::ReadAhead, &security, &backend)
            .require_posture(worth_store_io_scheduler::SecureIoPostureRequirement::ScopePreserving),
    )
    .expect("secure-frame read-ahead secure-I/O should admit");
    let work = work.with_secure_io_scope(secure_io);
    let budget = work.requested_budget();
    let policy = worth_store_io_scheduler::admit_queue_policy_receipt(work, policy_receipt(budget))
        .expect("policy receipt should bind the exact queue work");
    admit_queue_execution_plan(worth_store_io_scheduler::QueueExecutionAdmissionRequest::new(policy, &backend))
        .expect("secure-frame work should admit into a ready queue plan")
}

pub(crate) fn completion_for(
    plan: &QueueExecutionReadyPlan,
    scope: BackendQueueSpeculativeScope,
) -> BackendQueueExecutionCompletion {
    let witness = backend_witness();
    let posture = worth_store_physical_backend::BackendQueueExecutionPosture::from_admitted_capability(
        &witness,
        worth_store_physical_backend::BackendQueueExecutionAdaptation::None,
    )
    .expect("secure-frame backend posture should admit");
    worth_store_physical_backend::BackendQueueExecutionCompletion::for_certification(
        plan.backend_completion_binding()
            .backend_execution_binding(),
        posture,
    )
    .observe_queue_depth(1)
    .observe_read_ahead(1, scope)
}

pub(crate) fn secure_frame_backend(
    security: &IoSchedulerSecurityScopeAdmission,
) -> (
    worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission,
    AdmittedBackendCapabilityWitness,
) {
    let witness = backend_witness();
    let backend = admit_secure_frame_backend_capability_for_scheduler_claim(&witness, security)
        .expect("secure-frame backend should admit");
    (backend, witness)
}

pub(crate) fn backend_witness() -> AdmittedBackendCapabilityWitness {
    let request = worth_store_physical_backend::BackendCapabilityAdmissionRequest::new(
        worth_store_physical_backend::BackendTargetProfile::PosixFileFsyncDirSync,
        worth_store_physical_backend::BackendCapabilityEvidenceBasis::certified_backend_profile(),
        worth_store_physical_backend::BackendCapabilitySupportSet::all_supported(),
        worth_store_physical_backend::BackendMediaAssumptionSet::platform_file_defaults()
            .with_direct_io_alignment()
            .with_sector_atomicity()
            .with_page_cache_policy()
            .with_mmap_coherence()
            .with_async_ordering()
            .with_secure_frame_io()
            .with_flush_ordering()
            .with_fdatasync_durability(),
        worth_store_physical_backend::BackendRebindTriggers::kernel_filesystem_mount_firmware_and_backend(),
    );
    worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority::store_owned()
        .admit_backend_capability(request)
        .expect("secure-frame backend witness should admit")
}

pub(crate) fn io_qos_security_scope_admission() -> IoSchedulerSecurityScopeAdmission {
    let scope = worth_store_security::admitted_store_internal_security_scope_for_io_qos_test();
    admit_security_scope_for_scheduler(&scope).expect("scheduler security scope should admit")
}

pub(crate) fn speculative_scope(plan: &QueueExecutionReadyPlan) -> BackendQueueSpeculativeScope {
    worth_store_physical_backend::BackendQueueSpeculativeScope::admitted(
        plan.grouping_basis().security_scope_identity(),
        plan.grouping_basis().tenant_scope(),
        plan.grouping_basis().key_scope(),
    )
}

pub(crate) fn policy_receipt(budget: BackgroundResourceBudget) -> FoundationalPolicyAdmissionReceipt {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::DeltaBound)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("policy claim should build");
    performance()
        .policy_admission_receipt(claim)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
            breadth_units(budget),
            breadth_units(budget),
        )
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Locality,
            budget.read_ahead_window() as u32,
            budget.read_ahead_window() as u32,
        )
        .finish()
        .expect("policy receipt should build")
}

pub(crate) fn breadth_units(budget: BackgroundResourceBudget) -> u32 {
    (budget.queue_slots() + budget.worker_permits()) as u32
}