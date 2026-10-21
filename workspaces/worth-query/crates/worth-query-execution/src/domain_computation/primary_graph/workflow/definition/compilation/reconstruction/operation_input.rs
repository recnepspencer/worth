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
    let by_entity = nodes
        .iter()
        .map(|node| (node.entity, node))
        .collect::<HashMap<_, _>>();
    let mut inputs = HashMap::new();
    for connection in connections {
        if matches!(
            connection.kind.as_ref(),
            CompiledWorkflowConnectionKind::Data(
                ApplicationWorkflowDataFlow::CapturedOperationInput
            )
        ) {
            let source = by_entity.get(&connection.source);
            let target = by_entity.get(&connection.target);
            if !matches!(
                source.map(|node| node.kind()),
                Some(CompiledWorkflowNodeKind::Operation {
                    binding: Some(_),
                    requires_workflow_authority: false,
                    ..
                })
            ) || !matches!(
                target.map(|node| node.kind()),
                Some(CompiledWorkflowNodeKind::Operation {
                    binding: Some(_),
                    requires_workflow_authority: true,
                    ..
                })
            ) {
                return Err(WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                    "captured-operation-input endpoints are not bound control/guarded operations",
                ));
            }
        }
        if matches!(
            connection.kind.as_ref(),
            CompiledWorkflowConnectionKind::Data(
                ApplicationWorkflowDataFlow::OperationInput
                    | ApplicationWorkflowDataFlow::CapturedOperationInput
            )
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
