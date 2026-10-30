pub(crate) use worth_store_contracts::QueueProducerResourceShape;
pub(crate) use worth_store_test_support::harness::scheduling::scheduler_foreground_read_work;
pub(crate) use {
    worth_store_io_scheduler::admit_backend_capability_for_scheduler_claim,
    worth_store_io_scheduler::admit_queue_execution_plan,
    worth_store_io_scheduler::admit_secure_io_scope_for_scheduler,
    worth_store_io_scheduler::reject_lower_authority_secure_io_scope_source,
    worth_store_io_scheduler::IoSchedulerBackendCapabilityRequirement,
    worth_store_io_scheduler::QueueExecutionAdmissionRequest,
    worth_store_io_scheduler::QueueWorkDeclaration, worth_store_io_scheduler::SecureIoOperation,
    worth_store_io_scheduler::SecureIoPostureRequirement,
    worth_store_io_scheduler::SecureIoPreservationDenial,
    worth_store_io_scheduler::SecureIoPreservationRequest,
};
pub(crate) use {
    worth_store_security::classify_iam_role_as_security_scope_source,
    worth_store_security::classify_identity_provider_claim_as_security_scope_source,
    worth_store_security::classify_kms_key_id_as_security_scope_source,
    worth_store_security::classify_operator_identity_as_security_scope_source,
    worth_store_security::classify_terminal_json_label_as_security_scope_source,
};

pub(crate) use {
    crate::support::backend_witness, crate::support::point_read_budget,
    crate::support::scheduler_security_scope,
};

pub(crate) fn read_ahead_work(
    reservation: worth_store_io_scheduler::foreground_reservation::ForegroundReservationReceipt,
    budget: worth_store_io_scheduler::BackgroundResourceBudget,
) -> QueueWorkDeclaration {
    scheduler_foreground_read_work(
        reservation,
        11,
        worth_store_contracts::QueueProducerResourceShape::new()
            .with_queue_slots(budget.queue_slots())
            .with_bandwidth_tokens(budget.bandwidth_tokens())
            .with_read_ahead_windows(budget.read_ahead_window())
            .with_worker_permits(budget.worker_permits())
            .with_cache_residency_hints(budget.cache_residency_hints()),
    )
    .expect("scheduler-native foreground read work should lower")
}
