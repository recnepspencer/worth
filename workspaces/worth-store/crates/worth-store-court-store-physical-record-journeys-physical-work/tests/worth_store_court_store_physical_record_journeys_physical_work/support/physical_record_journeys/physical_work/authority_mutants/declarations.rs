pub(crate) use {worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalSignalSettlementOutcome, worth_store::physical_runtime::PhysicalWorkEffectFate};
pub(crate) use {worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write, worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_mutation_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

pub(crate) fn assert_cross_bound_receipts_are_rejected(predicate: &str) {
    let root = tempfile::tempdir().unwrap();
    let (profile, first_request, second_request) = disjoint_mutation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let first = admitted_write(&serving, first_request);
    let second = admitted_write(&serving, second_request);
    let before = serving.media_counters();

    let fates = serving
        .certification_cross_settle_physical_writes(
            worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(first, b"first001".as_slice()).unwrap(),
            worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(second, b"second02".as_slice()).unwrap(),
        )
        .unwrap();

    assert_eq!(
        fates,
        [
            worth_store::physical_runtime::PhysicalWorkEffectFate::StaleOrForeignOutcome,
            worth_store::physical_runtime::PhysicalWorkEffectFate::StaleOrForeignOutcome,
        ],
        "C5_PREDICATE:{predicate}: foreign backend receipts settled unrelated dispatched work"
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        2
    );
    assert!(serving.close_plan().execute().requires_inspection());
}