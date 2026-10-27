use worth_relational::facade::identity::{EntityId, PartitionId};

use super::{DecisionReadOutcome, DecisionReads, FailedDecisionRead};
use crate::domain_computation::primary_graph::application_attempt::{
    ComputationRead, WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationFactKey,
};

fn adjacency(slot: u64) -> WorthQueryApplicationFactKey {
    WorthQueryApplicationFactKey::Adjacency {
        relation: "owner".to_owned(),
        anchor: EntityId::new(PartitionId::main(), slot, 1),
        direction: WorthQueryApplicationAdjacencyDirection::Incoming,
        maximum_work_units: 2,
    }
}

#[test]
fn a_failed_read_an_owner_call_swallowed_seals_nothing() {
    let mut reads = DecisionReads::default();
    reads.record(adjacency(1), DecisionReadOutcome::Observed);
    let mut stronger = adjacency(1);
    if let WorthQueryApplicationFactKey::Adjacency {
        maximum_work_units, ..
    } = &mut stronger
    {
        *maximum_work_units = 19;
    }
    reads.record(stronger, DecisionReadOutcome::Observed);
    reads.reading = Some(ComputationRead::Membership);
    reads.record(adjacency(2), DecisionReadOutcome::Failed);
    reads.reading = None;
    reads.record(adjacency(3), DecisionReadOutcome::Failed);
    assert_eq!(
        reads
            .into_expected(&worth_execution::ExecutionRequest::serial(
                &crate::test_request_policy::serial_request()
            ))
            .err(),
        Some(FailedDecisionRead::Key(adjacency(2)))
    );
}

#[test]
fn observed_reads_seal_every_key() {
    let mut reads = DecisionReads::default();
    reads.record(adjacency(1), DecisionReadOutcome::Observed);
    let (keys, _) = reads
        .into_expected(&worth_execution::ExecutionRequest::serial(
            &crate::test_request_policy::serial_request(),
        ))
        .ok()
        .unwrap();
    assert_eq!(keys.iter().cloned().collect::<Vec<_>>(), [adjacency(1)]);
}

#[test]
fn shared_handler_and_computation_reads_keep_exact_keys_not_just_cardinality() {
    let mut reads = DecisionReads::default();
    reads.record(adjacency(1), DecisionReadOutcome::Observed);
    reads.record(adjacency(3), DecisionReadOutcome::Observed);
    let mut outgoing = adjacency(1);
    if let WorthQueryApplicationFactKey::Adjacency { direction, .. } = &mut outgoing {
        *direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    }
    let mut next_generation = adjacency(1);
    if let WorthQueryApplicationFactKey::Adjacency { anchor, .. } = &mut next_generation {
        *anchor = EntityId::new(PartitionId::main(), 1, 2);
    }
    reads.record(outgoing.clone(), DecisionReadOutcome::Observed);
    reads.record(next_generation.clone(), DecisionReadOutcome::Observed);
    reads.reading = Some(ComputationRead::Membership);
    let mut strongest = adjacency(1);
    if let WorthQueryApplicationFactKey::Adjacency {
        maximum_work_units, ..
    } = &mut strongest
    {
        *maximum_work_units = 19;
    }
    reads.record(strongest.clone(), DecisionReadOutcome::Observed);
    reads.record(adjacency(2), DecisionReadOutcome::Observed);
    reads.record(adjacency(2), DecisionReadOutcome::Observed);
    let (keys, _) = reads
        .into_expected(&worth_execution::ExecutionRequest::serial(
            &crate::test_request_policy::serial_request(),
        ))
        .ok()
        .unwrap();
    assert!(keys.attribution().contains_key(&strongest));
    assert!(!keys.attribution().contains_key(&adjacency(1)));
    let actual = std::collections::BTreeSet::from([
        strongest.clone(),
        outgoing,
        next_generation,
        adjacency(2),
        adjacency(3),
    ]);
    assert!(
        actual.iter().eq(keys.iter()),
        "shared key occurs once in order"
    );
    let mut wrong = actual.clone();
    wrong.remove(&adjacency(3));
    wrong.insert(adjacency(4));
    assert_eq!(wrong.len(), keys.iter().count());
    assert!(
        !wrong.iter().eq(keys.iter()),
        "same count cannot close another key set"
    );
}
