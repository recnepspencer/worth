//! Pure key authoring retains the actual normalization owner's original bill.
use super::super::{DecisionReadOutcome, DecisionReads};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::identity::{EntityId, PartitionId};
fn reads() -> DecisionReads {
    let mut reads = DecisionReads::default();
    for maximum_work_units in [1, 2] {
        reads.record(
            Key::Adjacency {
                relation: "historical-retention".into(),
                anchor: EntityId::new(PartitionId::main(), 1, 0),
                direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
                maximum_work_units,
            },
            DecisionReadOutcome::Observed,
        );
    }
    reads
}
#[test]
fn historical_key_aliases_retain_and_refund_the_original_authoring_payment() {
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let policy = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
    let execution = ExecutionRequest::serial(&policy);
    let baseline = budget.charged_memory_bytes();
    let (expected, computation) = reads().into_expected(&execution).unwrap();
    drop(computation);
    let charged = budget.charged_memory_bytes();
    assert!(
        charged > baseline,
        "the real conflicting-adjacency normalizer admits its backing"
    );
    let historical = expected.into_historical_keys();
    assert_eq!(budget.charged_memory_bytes(), charged);
    assert_eq!(historical.historical_handler_keys().count(), 1);
    let alias = Arc::clone(&historical);
    drop(historical);
    assert_eq!(
        budget.charged_memory_bytes(),
        charged,
        "the last alias owns the original bill"
    );
    assert_eq!(alias.historical_handler_keys().count(), 1);
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), baseline);
}
#[test]
fn refused_normalization_never_yields_historical_keys_or_a_retained_bill() {
    let budget = SerialMemoryBudget::new(Some(0));
    let policy = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
    let execution = ExecutionRequest::serial(&policy);
    assert!(matches!(
        reads().into_expected(&execution),
        Err(super::super::FailedDecisionRead::Execution(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
