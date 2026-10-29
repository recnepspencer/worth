pub(crate) use {tempfile::tempdir};
pub(crate) use {worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalStoreCloseOutcome, worth_store::physical_runtime::PhysicalStoreClosePhase, worth_store::physical_runtime::PhysicalWorkPreEffectDenial};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_mutation_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile};

pub(crate) fn write_commands(
    first: worth_store::physical_runtime::ResourceAdmittedPhysicalWork,
    second: worth_store::physical_runtime::ResourceAdmittedPhysicalWork,
) -> Box<[PhysicalExecutorCommand]> {
    vec![
        worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(first, b"first001".as_slice()).unwrap(),
        worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(second, b"second02".as_slice()).unwrap(),
    ]
    .into_boxed_slice()
}