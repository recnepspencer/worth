pub use tempfile::tempdir;
pub use worth_foundational::FoundationalPerformanceWorkClass;
pub use worth_proof::TransitionOutcome;
pub use {
    worth_store::physical_runtime::PhysicalEffectObligation,
    worth_store::physical_runtime::PhysicalSchedulerDemand,
    worth_store::physical_runtime::PhysicalSchedulerDenial,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::PhysicalWorkPreEffectDenial,
    worth_store::physical_runtime::PhysicalWorkReadiness,
};
pub use {
    worth_store_io_scheduler::admit_secure_io_scope_for_scheduler,
    worth_store_io_scheduler::admit_security_scope_for_scheduler,
    worth_store_io_scheduler::foreground_reservation::ForegroundIoLaneKind,
    worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission,
    worth_store_io_scheduler::QueueDurabilityClass,
    worth_store_io_scheduler::QueueExecutionAdmissionDenial,
    worth_store_io_scheduler::SecureIoOperation,
    worth_store_io_scheduler::SecureIoPreservationRequest,
};

pub use {
    crate::physical_record_journeys::physical_work::fixture::mutation_pair::foreground_saturation_fixture,
    crate::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    crate::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

pub use crate::physical_work::scheduler::policy_evidence::mismatched_policy_receipt;
pub use {
    crate::physical_work::scheduler::policy_evidence::exhausted_policy_receipt,
    crate::physical_work::scheduler::policy_evidence::policy_receipt,
    crate::physical_work::scheduler::policy_evidence::policy_receipt_for,
};

pub fn write_demand(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ready: worth_store::physical_runtime::ReadyPhysicalWork,
) -> PhysicalSchedulerDemand {
    worth_store::physical_runtime::PhysicalSchedulerDemand::foreground(
        ready,
        crate::physical_work::scheduler_reservation::reserved_page_write(serving),
        None,
    )
    .unwrap()
}

pub fn secure_demand(
    demand: PhysicalSchedulerDemand,
    backend: &IoSchedulerBackendCapabilityAdmission,
) -> PhysicalSchedulerDemand {
    let work = demand.queue_work();
    let admitted_scope = worth_store_security::admitted_security_scope_for_identity_for_test(
        work.security_scope_identity(),
    );
    let scheduler_security = admit_security_scope_for_scheduler(&admitted_scope)
        .expect("the physical-work fixture uses the scheduler's Store-internal security scope");
    let budget = work.requested_budget();
    let operation = if budget.read_ahead_window() > 0 {
        worth_store_io_scheduler::SecureIoOperation::ReadAhead
    } else if budget.write_back_window() > 0 {
        worth_store_io_scheduler::SecureIoOperation::WriteBack
    } else {
        worth_store_io_scheduler::SecureIoOperation::BatchedWrite
    };
    let secure_io = admit_secure_io_scope_for_scheduler(
        worth_store_io_scheduler::SecureIoPreservationRequest::new(
            operation,
            &scheduler_security,
            backend,
        ),
    )
    .expect("the physical-work fixture binds secure I/O to its exact scope and backend");
    demand.with_secure_io(secure_io)
}

pub fn ready_work(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalMutationWorkRequest,
) -> worth_store::physical_runtime::ReadyPhysicalWork {
    let receipt = match serving
        .physical_mutation_submission()
        .submit(request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("physical work should declare: {outcome:?}"),
    };
    let admitted = serving.admit_physical_work(receipt).unwrap();
    match serving.request_physical_work(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(blocked) => {
            panic!(
                "physical work unexpectedly blocked: {:?}",
                blocked.condition()
            )
        }
    }
}

pub fn ready_read_work(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalReadWorkRequest,
) -> worth_store::physical_runtime::ReadyPhysicalWork {
    let receipt = match serving
        .physical_read_submission()
        .submit(request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        outcome => panic!("physical work should declare: {outcome:?}"),
    };
    let admitted = serving.admit_physical_work(receipt).unwrap();
    match serving.request_physical_work(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(ready) => ready,
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(blocked) => {
            panic!(
                "physical work unexpectedly blocked: {:?}",
                blocked.condition()
            )
        }
    }
}
