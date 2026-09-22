use worth_relational::facade::identity::EntityId;

use super::{denial, transition_identity, SettledWorkflowTransition};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;

pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowTransitionReplay {
    node_path: String,
    identity: String,
    identity_bytes: [u8; 32],
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
}

pub(in crate::domain_computation::primary_graph) fn select_settled_replay_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    settled: SettledWorkflowTransition,
) -> Result<SelectedWorkflowTransitionReplay, WorthQueryApplicationAttemptDenial> {
    let node = compiled.node(settled.node()).ok_or_else(|| {
        denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "settled replay node is absent from the compiled node inventory",
        )
    })?;
    let (identity, identity_bytes) =
        transition_identity(compiled, instance, node, settled.occurrence())?;
    Ok(SelectedWorkflowTransitionReplay {
        node_path: node.path().to_owned(),
        identity,
        identity_bytes,
    })
}
