pub use {tempfile::tempdir};
pub use {worth_foundational::FoundationalPerformanceWorkClass};
pub use {worth_store::physical_runtime::PhysicalEffectObligation, worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalSchedulerDemand, worth_store::physical_runtime::PhysicalSignalSettlementOutcome, worth_store::physical_runtime::PhysicalStoreCloseOutcome, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkPreEffectDenial, worth_store::physical_runtime::PhysicalWorkPressureClass};
pub use {worth_store_physical_backend::MediaOperationRole};

pub use {crate::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_mutation_fixture, crate::physical_record_journeys::physical_work::fixture::semantic_work::family_locality_fixture, crate::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, crate::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};
pub use {crate::physical_work::scheduler::policy_evidence::policy_receipt, crate::physical_work::scheduler::policy_evidence::policy_receipt_for, crate::physical_record_journeys::physical_work::scheduler::ready_read_work, crate::physical_record_journeys::physical_work::scheduler::ready_work, crate::physical_record_journeys::physical_work::scheduler::secure_demand, crate::physical_record_journeys::physical_work::scheduler::write_demand};

pub fn admitted_write(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalMutationWorkRequest,
) -> worth_store::physical_runtime::ResourceAdmittedPhysicalWork {
    let demand = write_demand(serving, ready_work(serving, request));
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);
    serving
        .admit_physical_scheduler_demand(demand, &backend, policy_receipt(requested_budget))
        .unwrap()
}