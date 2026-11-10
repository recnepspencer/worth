//! Actual incoming-edge demand retains the original paged-symbol refusal.
use super::ring_source::RingRecipe;
use worth_relational::facade::{
    identity::KindId, storage::RelationalStorageUnavailableCause as Cause,
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_medium::immutable_members::ColdRead,
};
#[test]
fn the_remote_ring_label_stays_cold_until_its_incoming_edge_is_read() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = RingRecipe::new(execution, true).reopen(execution);
    let symbol = cold.remote_symbol.unwrap();

    let member = cold
        .directory
        .symbols
        .iter()
        .find(|page| page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last)
        .unwrap()
        .member;
    assert!(!cold.source.member_reads.lock().unwrap().contains(&member));
    let mut transaction = begin_main_transaction(&cold.runtime);
    transaction
        .read_entity(execution, cold.entities[0])
        .unwrap();

    assert!(!cold.source.member_reads.lock().unwrap().contains(&member));
    cold.source
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Member(member), Cause::MissingMember);
    let (_, basis) = cold
        .runtime
        .observe_branch(&cold.runtime.main_branch_identity())
        .unwrap();
    let view = cold
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let answer =
        view.bounded_incoming_relations_of_kind(execution, cold.entities[0], KindId(2), Some(32));
    assert!(
        matches!(
            answer,
            Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::StorageUnavailable(_))
        ),
        "incoming={answer:?}; symbol={symbol:?}; reads={:?}",
        cold.source.member_reads.lock().unwrap()
    );
    assert!(cold.source.member_reads.lock().unwrap().contains(&member));
}
