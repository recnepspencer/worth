pub(crate) use tempfile::tempdir;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::PhysicalEffectObligation,
    worth_store::physical_runtime::PhysicalExecutorCommand,
    worth_store::physical_runtime::PhysicalSchedulerDenial,
    worth_store::physical_runtime::PhysicalStoreCloseOutcome,
    worth_store::physical_runtime::PhysicalStoreClosePhase,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWorkExecutionOutcome,
    worth_store::physical_runtime::PhysicalWorkPreEffectDenial,
};
pub(crate) use {
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaFaultDirective,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_artifact_mutation_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::overlapping_mutation_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_work,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::write_demand,
    worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt,
};

pub(crate) fn assert_completed_distinct(
    first: &PhysicalWorkExecutionOutcome,
    second: &PhysicalWorkExecutionOutcome,
) {
    assert_eq!(
        first.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert_eq!(
        second.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert_ne!(
        first.settled().intent().identity(),
        second.settled().intent().identity()
    );
}
