use worth_relational::facade::identity::EntityId;

use super::{denial, transition_identity, SettledWorkflowTransition};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNodeKind,
};

pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowTransitionReplay {
    node_path: String,
    identity: String,
    identity_bytes: [u8; 32],
    terminal: bool,
}

impl SelectedWorkflowTransitionReplay {
    pub(in crate::domain_computation::primary_graph) fn identity(&self) -> &str {
        &self.identity
    }

    pub(in crate::domain_computation::primary_graph) const fn identity_bytes(&self) -> &[u8; 32] {
        &self.identity_bytes
    }

    pub(in crate::domain_computation::primary_graph) fn node_path(&self) -> &str {
        &self.node_path
    }

    pub(in crate::domain_computation::primary_graph) const fn terminal(&self) -> bool {
        self.terminal
    }
}

pub(in crate::domain_computation::primary_graph) fn select_settled_replay_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    settled: SettledWorkflowTransition,
    back_edge_iterations: &im::OrdMap<(EntityId, EntityId), u64>,
) -> Result<SelectedWorkflowTransitionReplay, WorthQueryApplicationAttemptDenial> {
    let node = compiled.node(settled.node()).ok_or_else(|| {
        denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "settled replay node is absent from the compiled node inventory",
        )
    })?;
    let (identity, identity_bytes) =
        transition_identity(
            compiled, instance, node, settled.occurrence(), back_edge_iterations,
            settled.outcome() == worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::NavigatedBack,
        )?;
    Ok(SelectedWorkflowTransitionReplay {
        node_path: node.path().to_owned(),
        identity,
        identity_bytes,
        terminal: matches!(node.kind(), CompiledWorkflowNodeKind::Terminal),
    })
}
