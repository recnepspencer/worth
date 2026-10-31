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
    home.physical_serving_for_test()
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
