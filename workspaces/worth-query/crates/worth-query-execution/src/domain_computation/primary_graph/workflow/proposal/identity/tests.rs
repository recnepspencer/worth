use worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector;
use worth_relational::facade::identity::{EntityId, PartitionId};

use super::{derive_workflow_proposal, WorkflowProposalCoverageMeaning};

#[test]
fn changing_one_subject_changes_only_its_coverage_identity() {
    let first_a = WorkflowProposalCoverageMeaning::new(
        ApplicationWorkflowSubjectSelector::Resource,
        EntityId::new(PartitionId::new(1), 10, 1),
    );
    let second_a = WorkflowProposalCoverageMeaning::new(
        ApplicationWorkflowSubjectSelector::Resource,
        EntityId::new(PartitionId::new(1), 11, 1),
    );
    let stable_b = WorkflowProposalCoverageMeaning::new(
        ApplicationWorkflowSubjectSelector::Related,
        EntityId::new(PartitionId::new(1), 20, 1),
    );
    let first = derive_workflow_proposal(
        "transition",
        "operation",
        "input",
        [1; 32],
        None,
        "proposal",
        vec![first_a.clone(), stable_b.clone()],
        EntityId::new(PartitionId::main(), 99, 1),
    );
    let second = derive_workflow_proposal(
        "transition",
        "operation",
        "input",
        [1; 32],
        None,
        "proposal",
        vec![second_a.clone(), stable_b.clone()],
        EntityId::new(PartitionId::main(), 99, 1),
    );

    assert_ne!(first.identity, second.identity);
    assert_ne!(first_a.identity, second_a.identity);
    assert_eq!(first.coverages[1].identity, second.coverages[1].identity);
}

#[test]
fn changing_shared_proposal_meaning_changes_each_coverage_scope() {
    let coverage = WorkflowProposalCoverageMeaning::new(
        ApplicationWorkflowSubjectSelector::Resource,
        EntityId::new(PartitionId::new(1), 10, 1),
    );
    let first = derive_workflow_proposal(
        "transition",
        "operation",
        "input",
        [1; 32],
        Some([2; 32]),
        "proposal",
        vec![coverage.clone()],
        EntityId::new(PartitionId::main(), 99, 1),
    );
    let changed = derive_workflow_proposal(
        "transition",
        "changed-operation",
        "input",
        [1; 32],
        Some([2; 32]),
        "proposal",
        vec![coverage],
        EntityId::new(PartitionId::main(), 99, 1),
    );
    let changed_input = derive_workflow_proposal(
        "transition",
        "operation",
        "input",
        [4; 32],
        Some([2; 32]),
        "proposal",
        vec![WorkflowProposalCoverageMeaning::new(
            ApplicationWorkflowSubjectSelector::Resource,
            EntityId::new(PartitionId::new(1), 10, 1),
        )],
        EntityId::new(PartitionId::main(), 99, 1),
    );
    let changed_source = derive_workflow_proposal(
        "transition",
        "operation",
        "input",
        [1; 32],
        Some([3; 32]),
        "proposal",
        vec![WorkflowProposalCoverageMeaning::new(
            ApplicationWorkflowSubjectSelector::Resource,
            EntityId::new(PartitionId::new(1), 10, 1),
        )],
        EntityId::new(PartitionId::main(), 99, 1),
    );

    assert_ne!(first.coverages[0].identity, changed.coverages[0].identity);
    assert_ne!(
        first.coverages[0].identity,
        changed_input.coverages[0].identity
    );
    assert_ne!(
        first.coverages[0].identity,
        changed_source.coverages[0].identity
    );
}

#[test]
fn changing_only_transition_occurrence_preserves_each_coverage_scope() {
    let coverage = WorkflowProposalCoverageMeaning::new(
        ApplicationWorkflowSubjectSelector::Resource,
        EntityId::new(PartitionId::new(1), 10, 1),
    );
    let first = derive_workflow_proposal(
        "transition/first-occurrence",
        "operation",
        "input",
        [1; 32],
        None,
        "proposal",
        vec![coverage.clone()],
        EntityId::new(PartitionId::main(), 99, 1),
    );
    let later = derive_workflow_proposal(
        "transition/later-occurrence",
        "operation",
        "input",
        [1; 32],
        None,
        "proposal",
        vec![coverage],
        EntityId::new(PartitionId::main(), 99, 1),
    );

    assert_ne!(first.identity, later.identity);
    assert_eq!(first.coverages[0].identity, later.coverages[0].identity);
}
