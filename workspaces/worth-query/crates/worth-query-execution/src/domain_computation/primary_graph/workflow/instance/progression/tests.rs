use super::*;
use worth_relational::facade::identity::PartitionId;

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::new(7), slot, 1)
}

fn completed(node: EntityId, occurrence: u64) -> SettledWorkflowTransition {
    SettledWorkflowTransition::new(
        node,
        occurrence,
        ApplicationWorkflowControlOutcome::Completed,
        None,
    )
}

#[test]
fn out_of_order_history_reconstructs_one_incremental_head() {
    let start = entity(10);
    let middle = entity(11);
    let terminal = entity(12);
    let edges = BTreeMap::from([(start, middle), (middle, terminal)]);
    let mut history = [completed(middle, 1), completed(start, 0)];

    let progress =
        WorkflowInstanceProgress::reconstruct_with(&mut history, start, |source, _, _| {
            edges
                .get(&source)
                .copied()
                .ok_or_else(|| denial("missing successor"))
        })
        .expect("the immutable history selects its compiled terminal");

    assert_eq!(progress.head(), terminal);
    assert_eq!(progress.next_occurrence(), 2);
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
        assert!(
            WorkflowInstanceProgress::reconstruct_with(&mut history, start, |source, _, _| edges
                .get(&source)
                .copied()
                .ok_or_else(|| denial("missing successor")),)
            .is_err()
        );
    }
}

#[test]
fn retry_attempt_counts_advance_without_rescanning_history() {
    let start = entity(30);
    let mut history = [completed(start, 0), completed(start, 1)];
    let mut observed_attempts = Vec::new();
    WorkflowInstanceProgress::reconstruct_with(&mut history, start, |source, _, attempts| {
        observed_attempts.push(attempts);
        Ok(source)
    })
    .expect("retry history is contiguous");

    assert_eq!(observed_attempts, [1, 2]);
}

#[test]
fn completed_history_requires_the_terminal_settlement_to_be_excluded() {
    let start = entity(40);
    let terminal = entity(41);
    let edges = BTreeMap::from([(start, terminal)]);
    let mut untrimmed = [completed(start, 0), completed(terminal, 1)];

    assert!(
        WorkflowInstanceProgress::reconstruct_with(&mut untrimmed, start, |source, _, _| edges
            .get(&source)
            .copied()
            .ok_or_else(|| denial("settled terminal has no successor")),)
        .is_err()
    );
}

#[test]
fn latest_transition_locator_tracks_occurrence_without_becoming_authority() {
    let node = entity(50);
    let mut progress = WorkflowInstanceProgress {
        head: node,
        next_occurrence: 2,
        retry_counts: OrdMap::new(),
        path_depth: 0,
        path: OrdMap::new(),
        back_blocked_by_operation: false,
        back_edge_iterations: OrdMap::new(),
        latest_transitions: OrdMap::new(),
        latest_transition_identities: OrdMap::new(),
        latest_assessment_evidence: OrdMap::new(),
    };
    let latest = WorkflowTransitionLocator::new(entity(52), completed(node, 1));
    progress.retain_observation(WorkflowTransitionProgressObservation::new(latest, None));
    progress.retain_observation(WorkflowTransitionProgressObservation::new(
        WorkflowTransitionLocator::new(entity(51), completed(node, 0)),
        None,
    ));

    assert_eq!(progress.latest_transition(node), Some(latest));
}

#[test]
fn evidence_locator_survives_a_later_reuse_transition_without_new_evidence() {
    let node = entity(60);
    let mut progress = WorkflowInstanceProgress {
        head: node,
        next_occurrence: 2,
        retry_counts: OrdMap::new(),
        path_depth: 0,
        path: OrdMap::new(),
        back_blocked_by_operation: false,
        back_edge_iterations: OrdMap::new(),
        latest_transitions: OrdMap::new(),
        latest_transition_identities: OrdMap::new(),
        latest_assessment_evidence: OrdMap::new(),
    };
    let evidence_transition = WorkflowTransitionLocator::new(entity(61), completed(node, 0));
    progress.retain_observation(WorkflowTransitionProgressObservation::new(
        evidence_transition,
        Some(entity(62)),
    ));
    progress.retain_observation(WorkflowTransitionProgressObservation::new(
        WorkflowTransitionLocator::new(entity(63), completed(node, 1)),
        None,
    ));

    let retained = progress
        .latest_assessment_evidence(node)
        .expect("reused evidence remains locatable");
    assert_eq!(retained.transition(), evidence_transition);
    assert_eq!(retained.evidence(), entity(62));
}

#[test]
fn back_reconstructs_the_same_head_and_iteration_from_settlements() {
    let first = entity(70);
    let second = entity(71);
    let third = entity(72);
    let mut history = [
        completed(second, 1),
        SettledWorkflowTransition::new(
            third,
            2,
            ApplicationWorkflowControlOutcome::NavigatedBack,
            None,
        ),
        completed(first, 0),
    ];
    let progress = WorkflowInstanceProgress::reconstruct_with(
        &mut history,
        first,
        |source, _, _| match source {
            node if node == first => Ok(second),
            node if node == second => Ok(third),
            _ => Err(denial("unexpected forward source")),
        },
    )
    .expect("Back follows the settled predecessor path");
    assert_eq!(progress.head(), second);
    assert_eq!(progress.next_occurrence(), 3);
    assert_eq!(
        progress.back_edge_iterations().get(&(third, second)),
        Some(&1)
    );
}

#[test]
fn back_cannot_cross_a_performed_operation_or_invent_a_predecessor() {
    let first = entity(80);
    let second = entity(81);
    let back = SettledWorkflowTransition::new(
        second,
        1,
        ApplicationWorkflowControlOutcome::NavigatedBack,
        None,
    );
    let mut without_predecessor = [SettledWorkflowTransition::new(
        first,
        0,
        ApplicationWorkflowControlOutcome::NavigatedBack,
        None,
    )];
    assert!(WorkflowInstanceProgress::reconstruct_with(
        &mut without_predecessor,
        first,
        |_, _, _| Ok(second),
    )
    .is_err());
    let mut through_operation = [
        SettledWorkflowTransition::new(
            first,
            0,
            ApplicationWorkflowControlOutcome::Completed,
            Some([9; 32]),
        ),
        back,
    ];
    assert!(WorkflowInstanceProgress::reconstruct_with(
        &mut through_operation,
        first,
        |_, _, _| Ok(second),
    )
    .is_err());
}
