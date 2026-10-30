pub(crate) use worth_store_budgets::CounterEvidenceStrength;
pub(crate) use worth_store_certification::S6LatencyInterferenceEvidence;
pub(crate) use worth_store_contracts::QueueProducerResourceShape;
pub(crate) use worth_store_io_scheduler::foreground_reservation::admitted_point_read_reservation_for_certification_test;
pub(crate) use {
    worth_foundational::performance,
    worth_foundational::FoundationalPerformanceAccessPatternPosture,
    worth_foundational::FoundationalPerformanceBoundary,
    worth_foundational::FoundationalPerformanceBreadthLocalityPosture,
    worth_foundational::FoundationalPerformanceBudgetKind,
    worth_foundational::FoundationalPerformanceEvidenceStrength,
    worth_foundational::FoundationalPerformanceExecutionTemperature,
    worth_foundational::FoundationalPerformanceFallbackDebtPosture,
    worth_foundational::FoundationalPerformanceFreshnessRetentionPosture,
    worth_foundational::FoundationalPerformanceWorkClass,
    worth_foundational::FoundationalPolicyAdmissionReceipt,
};
pub(crate) use {
    worth_store_io_scheduler::admit_backend_capability_for_scheduler_claim,
    worth_store_io_scheduler::admit_queue_execution_plan,
    worth_store_io_scheduler::admit_secure_io_scope_for_scheduler,
    worth_store_io_scheduler::admit_security_scope_for_scheduler,
    worth_store_io_scheduler::assess_queue_latency_envelope,
    worth_store_io_scheduler::execute_ready_queue_plan,
    worth_store_io_scheduler::BackgroundResourceBudget,
    worth_store_io_scheduler::CacheResidencyHint,
    worth_store_io_scheduler::InterferenceCounterName,
    worth_store_io_scheduler::InterferenceCounterRequirement,
    worth_store_io_scheduler::LatencyEnvelopeClaim,
    worth_store_io_scheduler::QueueExecutionReadyPlan, worth_store_io_scheduler::QueueSlot,
    worth_store_io_scheduler::ReadAheadWindow, worth_store_io_scheduler::SecureIoOperation,
    worth_store_io_scheduler::SecureIoPostureRequirement,
    worth_store_io_scheduler::SecureIoPreservationRequest, worth_store_io_scheduler::WorkerPermit,
};
pub(crate) use {
    worth_store_physical_backend::AdmittedBackendCapabilityWitness,
    worth_store_physical_backend::BackendCapabilityAdmissionRequest,
    worth_store_physical_backend::BackendCapabilityEvidenceBasis,
    worth_store_physical_backend::BackendCapabilitySupportSet,
    worth_store_physical_backend::BackendMediaAssumptionSet,
    worth_store_physical_backend::BackendQueueExecutionAdaptation,
    worth_store_physical_backend::BackendQueueExecutionCompletion,
    worth_store_physical_backend::BackendQueueExecutionPosture,
    worth_store_physical_backend::BackendQueueSpeculativeScope,
    worth_store_physical_backend::BackendRebindTriggers,
    worth_store_physical_backend::BackendTargetProfile,
    worth_store_physical_backend::PhysicalBackendCapabilityAdmissionAuthority,
};

pub(crate) fn certified_latency_evidence(queue_depth: u32) -> S6LatencyInterferenceEvidence {
    let plan = admitted_read_ahead_plan();
    let replay_identity = plan.replay_identity().clone();
    let lane = plan.work().class();
    let completion = completion_for(&plan, queue_depth);
    let outcome = execute_ready_queue_plan(plan, completion);
    let claim = worth_store_io_scheduler::LatencyEnvelopeClaim::for_queue_execution(
        replay_identity,
        "s6-profile/posix-file",
        lane,
    )
    .require_counter(worth_store_io_scheduler::InterferenceCounterRequirement::queue_depth());
    let assessment = assess_queue_latency_envelope(&claim, &outcome)
        .expect("real queue execution should assess latency evidence");

    worth_store_certification::S6LatencyInterferenceEvidence::from_assessment(&assessment)
        .expect("scheduler assessment should certify")
}

pub(crate) fn receipt_rows(evidence: &S6LatencyInterferenceEvidence) -> Vec<(String, u64)> {
    evidence
        .counter_backed_receipt()
        .counter_rows()
        .iter()
        .map(|row| (row.name().as_str().to_owned(), row.observed_count()))
        .collect()
}

pub(crate) fn admitted_read_ahead_plan() -> QueueExecutionReadyPlan {
    let reservation = admitted_point_read_reservation_for_certification_test();
    let budget = point_read_budget();
    let work = worth_store_test_support::harness::scheduling::scheduler_foreground_read_work(
        reservation,
        7,
        worth_store_contracts::QueueProducerResourceShape::new()
            .with_queue_slots(budget.queue_slots())
            .with_read_ahead_windows(budget.read_ahead_window())
            .with_worker_permits(budget.worker_permits())
            .with_cache_residency_hints(budget.cache_residency_hints()),
    )
    .expect("scheduler-native foreground read should lower to queue work");
    let backend = admit_backend_capability_for_scheduler_claim(
        &backend_witness(),
        work.backend_requirement(),
    )
    .expect("backend should admit test queue work");
    let security = io_qos_security_scope_admission();
    let secure_io = admit_secure_io_scope_for_scheduler(
        worth_store_io_scheduler::SecureIoPreservationRequest::new(
            worth_store_io_scheduler::SecureIoOperation::ReadAhead,
            &security,
            &backend,
        )
        .require_posture(worth_store_io_scheduler::SecureIoPostureRequirement::ScopePreserving),
    )
    .expect("read-ahead secure-I/O should admit");
    let work = work.with_secure_io_scope(secure_io);
    let policy = worth_store_io_scheduler::admit_queue_policy_receipt(work, policy_receipt(budget))
        .expect("policy receipt should bind the exact queue work");
    admit_queue_execution_plan(
        worth_store_io_scheduler::QueueExecutionAdmissionRequest::new(policy, &backend),
    )
    .expect("queue work should admit")
}

pub(crate) fn completion_for(
    plan: &QueueExecutionReadyPlan,
    queue_depth: u32,
) -> BackendQueueExecutionCompletion {
    let witness = backend_witness();
    let posture =
        worth_store_physical_backend::BackendQueueExecutionPosture::from_admitted_capability(
            &witness,
            worth_store_physical_backend::BackendQueueExecutionAdaptation::None,
        )
        .expect("backend posture should admit");
    worth_store_physical_backend::BackendQueueExecutionCompletion::for_certification(
        plan.backend_completion_binding()
            .backend_execution_binding(),
        posture,
    )
    .observe_queue_depth(queue_depth)
    .observe_read_ahead(1, speculative_scope(plan))
}

pub(crate) fn speculative_scope(plan: &QueueExecutionReadyPlan) -> BackendQueueSpeculativeScope {
    worth_store_physical_backend::BackendQueueSpeculativeScope::admitted(
        plan.grouping_basis().security_scope_identity(),
        plan.grouping_basis().tenant_scope(),
        plan.grouping_basis().key_scope(),
    )
}

pub(crate) fn point_read_budget() -> BackgroundResourceBudget {
    worth_store_io_scheduler::BackgroundResourceBudget::new()
        .with_queue_slots(worth_store_io_scheduler::QueueSlot::new(1).unwrap())
        .with_read_ahead(worth_store_io_scheduler::ReadAheadWindow::pages(1).unwrap())
        .with_worker_permits(worth_store_io_scheduler::WorkerPermit::new(1).unwrap())
        .with_cache_residency(worth_store_io_scheduler::CacheResidencyHint::frames(1).unwrap())
}

pub(crate) fn point_read_units(budget: BackgroundResourceBudget) -> u64 {
    budget
        .queue_slots()
        .saturating_add(budget.read_ahead_window())
        .saturating_add(budget.worker_permits())
        .saturating_add(budget.cache_residency_hints())
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
        .expect("backend witness should admit")
}

pub(crate) fn io_qos_security_scope_admission(
) -> worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission {
    let scope = worth_store_security::admitted_store_internal_security_scope_for_io_qos_test();
    admit_security_scope_for_scheduler(&scope).expect("scheduler security scope should admit")
}

pub(crate) fn policy_receipt(
    budget: BackgroundResourceBudget,
) -> FoundationalPolicyAdmissionReceipt {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission,
        )
        .breadth_locality(
            worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal,
        )
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal,
        )
        .execution_temperature(
            worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
        )
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("policy claim should build");
    performance()
        .policy_admission_receipt(claim)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
            budget.queue_slots().saturating_add(budget.worker_permits()) as u32,
            budget.queue_slots().saturating_add(budget.worker_permits()) as u32,
        )
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Density,
            budget.cache_residency_hints() as u32,
            budget.cache_residency_hints() as u32,
        )
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Locality,
            budget.read_ahead_window() as u32,
            budget.read_ahead_window() as u32,
        )
        .finish()
        .expect("policy receipt should build")
}
