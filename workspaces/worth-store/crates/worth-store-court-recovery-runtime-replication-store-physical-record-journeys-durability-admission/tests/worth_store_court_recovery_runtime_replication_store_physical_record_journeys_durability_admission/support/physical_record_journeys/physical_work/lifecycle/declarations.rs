pub(crate) use std::collections::BTreeSet;

pub(crate) use tempfile::tempdir;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::PhysicalSignalShutdownOutcome,
    worth_store::physical_runtime::PhysicalWorkSubmissionDenial,
    worth_store::physical_runtime::PhysicalWorkSubmissionOutcome,
    worth_store::physical_runtime::PhysicalWorkSubmissionStale,
    worth_store::physical_runtime::PhysicalWorkTerminalDisposition,
    worth_store::physical_runtime::PhysicalWorkTerminalStage,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_mutation_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

pub(crate) fn success(
    outcome: PhysicalWorkSubmissionOutcome,
) -> worth_store::physical_runtime::PhysicalWorkSubmissionReceipt {
    match outcome.into_raw() {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        _ => panic!("physical work submission should succeed"),
    }
}
