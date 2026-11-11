//! Physical membership, rather than encoded-byte estimates, proves reuse.
pub(crate) use worth_execution::ExecutionRequest;
pub(crate) use worth_store_application::ReadmittedApplicationHome;
pub(crate) use {
    worth_relational::facade::history::BranchId, worth_relational::facade::identity::PartitionId,
    worth_relational::facade::transactions::CreateIntent,
    worth_relational::facade::transactions::MutationIntent,
};
pub(crate) use {
    worth_store::physical_runtime::AnchoredRecordMemberLocator,
    worth_store::physical_runtime::AnchoredRecordName,
};

pub(crate) use {crate::batches, crate::durable_run, crate::workload};

pub(crate) fn members(home: &ReadmittedApplicationHome) -> Vec<AnchoredRecordMemberLocator> {
    read_members(home.physical_serving_for_test())
}

pub(crate) fn members_of_world(
    fixture: &worth_store_testkit_application_oracle::facade::world_home::WorldHomeFixture,
) -> Vec<AnchoredRecordMemberLocator> {
    read_members(fixture.home.physical_serving_for_test())
}

fn read_members(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) -> Vec<AnchoredRecordMemberLocator> {
    serving
        .anchored_records()
        .unwrap()
        .read(
            worth_store::physical_runtime::AnchoredRecordName::new(
                crate::selected_checkpoint::ANCHOR,
            )
            .unwrap(),
            1 << 30,
        )
        .unwrap()
        .unwrap()
        .members()
        .to_vec()
}
