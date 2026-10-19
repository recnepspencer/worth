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
    expected_identity: &str,
) -> Result<SelectedWorkflowTransitionReplay, WorthQueryApplicationAttemptDenial> {
    let node = compiled.node(settled.node()).ok_or_else(|| {
        denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "settled replay node is absent from the compiled node inventory",
        )
    })?;
    let (identity, identity_bytes) = if settled.outcome() == worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Terminated {
        if settled.operation_receipt_identity().is_none() {
            return Err(denial(WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch, "business termination history has no operation receipt"));
        }
        let mut matching = None;
        for action in compiled.global_actions() {
            let candidate = super::identity::business_termination_identity(compiled, instance, node, settled.occurrence(), back_edge_iterations, action)?;
            if candidate.0 == expected_identity {
                if matching.is_some() { return Err(denial(WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch, "termination history has ambiguous binding")); }
                matching = Some(candidate);
            }
        }
        matching.ok_or_else(|| denial(WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch, "termination history matches no declared action"))?
    } else if settled.outcome() == worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Returned {
        if settled.operation_receipt_identity().is_none() {
            return Err(denial(WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch, "business return history has no operation receipt"));
        }
        super::identity::business_return_identity(compiled, instance, node, settled.occurrence(), back_edge_iterations)?
    } else {
        transition_identity(
            compiled, instance, node, settled.occurrence(), back_edge_iterations,
            settled.outcome() == worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::NavigatedBack,
        )?
    };
    Ok(SelectedWorkflowTransitionReplay {
        node_path: node.path().to_owned(),
        identity,
        identity_bytes,
        terminal: matches!(node.kind(), CompiledWorkflowNodeKind::Terminal) || settled.outcome() == worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Terminated,
    })
}
