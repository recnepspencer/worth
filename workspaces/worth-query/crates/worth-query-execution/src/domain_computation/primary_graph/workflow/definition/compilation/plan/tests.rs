use super::{
    CompiledWorkflowConnection, CompiledWorkflowConnectionKind, CompiledWorkflowNode,
    CompiledWorkflowNodeKind, CompiledWorkflowNodeMeaning,
};
use std::sync::Arc;
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn compiled_identity_material_carries_full_record_coordinates() {
    let first_node = CompiledWorkflowNode {
        entity: EntityId::new(PartitionId::new(1), 10, 3),
        meaning: Arc::new(CompiledWorkflowNodeMeaning {
            path: "apply".to_owned(),
            kind: CompiledWorkflowNodeKind::Terminal,
        }),
    };
    let reassociated_node = CompiledWorkflowNode {
        entity: EntityId::new(PartitionId::new(2), 10, 3),
        meaning: Arc::new(CompiledWorkflowNodeMeaning {
            path: "apply".to_owned(),
            kind: CompiledWorkflowNodeKind::Terminal,
        }),
    };

    assert_ne!(
        first_node.identity_material(),
        reassociated_node.identity_material()
    );

    let first_connection = CompiledWorkflowConnection {
        entity: EntityId::new(PartitionId::new(1), 20, 3),
        source: first_node.entity,
        target: EntityId::new(PartitionId::new(1), 11, 3),
        kind: Arc::new(CompiledWorkflowConnectionKind::Control(
            ApplicationWorkflowControlOutcome::Completed,
        )),
    };
    let reassociated_connection = CompiledWorkflowConnection {
        entity: first_connection.entity,
        source: reassociated_node.entity,
        target: first_connection.target,
        kind: Arc::new(CompiledWorkflowConnectionKind::Control(
            ApplicationWorkflowControlOutcome::Completed,
        )),
    };

    assert_ne!(
        first_connection.identity_material(),
        reassociated_connection.identity_material()
    );
}
