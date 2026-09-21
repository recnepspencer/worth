use std::collections::BTreeMap;

use super::{resolve_head_entity, unique_successor_entity, SettledWorkflowTransition};
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, PartitionId};

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::new(7), slot, 1)
}

fn completed(node: EntityId, occurrence: u64) -> SettledWorkflowTransition {
    SettledWorkflowTransition::new(
        node,
        occurrence,
        ApplicationWorkflowControlOutcome::Completed,
    )
}

fn linear_successor(
    edges: &BTreeMap<EntityId, EntityId>,
    source: EntityId,
    _: ApplicationWorkflowControlOutcome,
) -> Result<EntityId, super::WorthQueryApplicationAttemptDenial> {
    edges.get(&source).copied().ok_or_else(|| {
        super::denial(
            super::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "missing test successor",
        )
    })
}

#[test]
fn out_of_order_settlements_reconstruct_the_exact_terminal_head() {
    let start = entity(10);
    let middle = entity(11);
    let terminal = entity(12);
    let edges = BTreeMap::from([(start, middle), (middle, terminal)]);
    let mut history = [completed(middle, 1), completed(start, 0)];

    let head = resolve_head_entity(&mut history, start, |source, outcome| {
        linear_successor(&edges, source, outcome)
    })
    .expect("the immutable history selects its compiled terminal");

    assert_eq!(head, terminal);
}

#[test]
fn duplicate_gap_and_wrong_node_histories_fail_closed() {
    let start = entity(20);
    let middle = entity(21);
    let terminal = entity(22);
    let edges = BTreeMap::from([(start, middle), (middle, terminal)]);
    for mut history in [
        vec![completed(start, 0), completed(middle, 0)],
        vec![completed(start, 0), completed(middle, 2)],
        vec![completed(entity(99), 0)],
    ] {
        assert!(resolve_head_entity(&mut history, start, |source, outcome| {
            linear_successor(&edges, source, outcome)
        })
        .is_err());
    }
}

#[test]
fn missing_and_ambiguous_successors_fail_closed() {
    assert!(unique_successor_entity([]).is_err());
    assert!(unique_successor_entity([entity(31), entity(32)]).is_err());
    assert_eq!(
        unique_successor_entity([entity(33)]).expect("one successor is exact"),
        entity(33)
    );
}
