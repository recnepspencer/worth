//! Current guarded-input admission for native reconstructed definition facts.

use std::collections::HashMap;
use worth_query_declaration::facade::application_program::ApplicationWorkflowDataFlow;

use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    },
    workflow::definition::compilation::plan::{
        CompiledWorkflowConnection, CompiledWorkflowConnectionKind, CompiledWorkflowNode,
        CompiledWorkflowNodeKind,
    },
};

/// O(nodes + connections) cold work; never re-proved by warm dispatch.
pub(super) fn admit(
    nodes: &[CompiledWorkflowNode],
    connections: &[CompiledWorkflowConnection],
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let mut inputs = HashMap::new();
    for connection in connections {
        if matches!(
            connection.kind.as_ref(),
            CompiledWorkflowConnectionKind::Data(ApplicationWorkflowDataFlow::OperationInput)
        ) {
            *inputs.entry(connection.target).or_insert(0_usize) += 1;
        }
    }
    for node in nodes {
        if matches!(
            node.kind(),
            CompiledWorkflowNodeKind::Operation {
                requires_workflow_authority: true,
                ..
            }
        ) && inputs.get(&node.entity).copied().unwrap_or(0) != 1
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionMissingOperationInput,
                node.path(),
            ));
        }
    }
    Ok(())
}
