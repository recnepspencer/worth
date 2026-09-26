use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome as Outcome;
use worth_query_installation::facade::WorthQueryWorkflowNodeDependency;
use worth_relational::facade::identity::{EntityId, PartitionId};

use super::super::super::definition::{WorkflowDefinitionDependencies, WorkflowRetainedNode};
use super::super::super::instance::WorkflowInventoriedTransition;
use super::*;

const APPROVAL: EntityId = EntityId::new(PartitionId(0), 1, 0);
const OPERATION: EntityId = EntityId::new(PartitionId(0), 2, 0);
const TERMINAL: EntityId = EntityId::new(PartitionId(0), 3, 0);

fn approval_gated() -> WorkflowDefinitionDependencies {
    WorkflowDefinitionDependencies {
        spec: "spec".to_owned(),
        nodes: vec![
            WorkflowRetainedNode {
                entity: APPROVAL,
                path: "approve".to_owned(),
                dependency: Some(WorthQueryWorkflowNodeDependency::Approval {
                    identifier: "approve".to_owned(),
                    capability_type: "capability".to_owned(),
                    operation: "ship".to_owned(),
                    capability_identity: "00".to_owned(),
                }),
            },
            WorkflowRetainedNode {
                entity: OPERATION,
                path: "ship".to_owned(),
                dependency: Some(WorthQueryWorkflowNodeDependency::Operation {
                    identifier: "ship".to_owned(),
                    input_type: "input".to_owned(),
                    binding: Some("ship-binding".to_owned()),
                    requires_authority: true,
                }),
            },
            WorkflowRetainedNode {
                entity: TERMINAL,
                path: "done".to_owned(),
                dependency: None,
            },
        ],
        approval_authorities: vec![(APPROVAL, OPERATION)],
    }
}

const fn transition(
    node: EntityId,
    occurrence: u64,
    outcome: Outcome,
    receipted: bool,
) -> WorkflowInventoriedTransition {
    WorkflowInventoriedTransition {
        node,
        occurrence,
        outcome,
        receipted,
    }
}

#[test]
fn fresh_instance_holds_no_custody() {
    assert_eq!(
        instance_custody(&approval_gated(), &[]),
        Custody::Unperformed
    );
}

#[test]
fn approved_operation_awaiting_settlement_is_outstanding() {
    let transitions = [transition(APPROVAL, 1, Outcome::Approved, false)];
    assert_eq!(
        instance_custody(&approval_gated(), &transitions),
        Custody::ApprovalOutstanding {
            approval_node_path: "approve".to_owned(),
        }
    );
}

#[test]
fn a_receipt_before_the_latest_approval_does_not_settle_it() {
    let transitions = [
        transition(APPROVAL, 1, Outcome::Approved, false),
        transition(OPERATION, 2, Outcome::Completed, true),
        transition(APPROVAL, 3, Outcome::Approved, false),
    ];
    assert!(matches!(
        instance_custody(&approval_gated(), &transitions),
        Custody::ApprovalOutstanding { .. }
    ));
}

#[test]
fn a_receipted_operation_after_approval_settles_it() {
    let transitions = [
        transition(APPROVAL, 1, Outcome::Approved, false),
        transition(OPERATION, 2, Outcome::Completed, true),
    ];
    assert_eq!(
        instance_custody(&approval_gated(), &transitions),
        Custody::Performed
    );
}

#[test]
fn a_rejected_approval_is_not_outstanding() {
    let transitions = [transition(APPROVAL, 1, Outcome::Rejected, false)];
    assert_eq!(
        instance_custody(&approval_gated(), &transitions),
        Custody::Unperformed
    );
}

#[test]
fn definitions_can_always_retire_and_carry_only_when_compatible() {
    let incompatible = Compatibility::Incompatible(Incompatibility::VocabularyUnsupported {
        spec: "spec".to_owned(),
    });
    assert_eq!(
        definition_dispositions(&Compatibility::Compatible),
        [DefinitionDisposition::Carry, DefinitionDisposition::Retire]
    );
    assert_eq!(
        definition_dispositions(&incompatible),
        [DefinitionDisposition::Retire]
    );
}

#[test]
fn instance_law_covers_every_compatibility_and_custody() {
    let incompatible = Compatibility::Incompatible(Incompatibility::NodeUncovered {
        node_path: "ship".to_owned(),
    });
    let outstanding = Custody::ApprovalOutstanding {
        approval_node_path: "approve".to_owned(),
    };
    let compatible = Compatibility::Compatible;
    assert_eq!(
        instance_dispositions(&compatible, &Custody::Unperformed),
        [InstanceDisposition::Carry, InstanceDisposition::Cancel]
    );
    assert_eq!(
        instance_dispositions(&compatible, &Custody::Performed),
        [InstanceDisposition::Carry]
    );
    assert!(instance_dispositions(&compatible, &outstanding).is_empty());
    assert_eq!(
        instance_dispositions(&incompatible, &Custody::Unperformed),
        [InstanceDisposition::Cancel]
    );
    assert!(instance_dispositions(&incompatible, &Custody::Performed).is_empty());
    assert!(instance_dispositions(&incompatible, &outstanding).is_empty());
}
