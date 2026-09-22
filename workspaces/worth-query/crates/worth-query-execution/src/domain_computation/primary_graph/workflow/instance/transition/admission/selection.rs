use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::EntityId;

use super::SettledWorkflowTransition;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
};

mod approval;
mod identity;
mod navigation;
mod replay;
use approval::select_approval;
pub(in crate::domain_computation::primary_graph) use approval::SelectedWorkflowApproval;
use identity::transition_identity;
use navigation::unique_successor;
pub(in crate::domain_computation::primary_graph) use replay::select_settled_replay_transition;
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowTransition {
    pub(super) node: EntityId,
    pub(super) node_path: String,
    pub(super) occurrence: u64,
    pub(super) identity: String,
    pub(super) identity_bytes: [u8; 32],
    kind: SelectedWorkflowTransitionKind,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) enum SelectedWorkflowTransitionKind {
    Operation(SelectedWorkflowOperation),
    Assessment(SelectedWorkflowAssessment),
    Condition(SelectedWorkflowCondition),
    Approval(SelectedWorkflowApproval),
    EvidenceJoin(
        worth_query_declaration::facade::application_program::ApplicationWorkflowEvidenceJoinPolicy,
    ),
    Terminal,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowOperation {
    pub(in crate::domain_computation::primary_graph) operation: String,
    pub(in crate::domain_computation::primary_graph) input_type: String,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowAssessment {
    pub(in crate::domain_computation::primary_graph) query: String,
    pub(in crate::domain_computation::primary_graph) parameter_type: String,
    pub(in crate::domain_computation::primary_graph) result_type: String,
    pub(in crate::domain_computation::primary_graph) binding: String,
    pub(in crate::domain_computation::primary_graph) subject:
        worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowCondition {
    pub(in crate::domain_computation::primary_graph) query: String,
    pub(in crate::domain_computation::primary_graph) parameter_type: String,
    pub(in crate::domain_computation::primary_graph) result_type: String,
    pub(in crate::domain_computation::primary_graph) binding: String,
}

impl SelectedWorkflowTransition {
    pub(in crate::domain_computation::primary_graph) const fn node(&self) -> EntityId {
        self.node
    }

    pub(in crate::domain_computation::primary_graph) fn identity(&self) -> &str {
        &self.identity
    }

    pub(in crate::domain_computation::primary_graph) const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    pub(in crate::domain_computation::primary_graph) fn node_path(&self) -> &str {
        &self.node_path
    }

    pub(in crate::domain_computation::primary_graph) const fn kind(
        &self,
    ) -> &SelectedWorkflowTransitionKind {
        &self.kind
    }

    pub(in crate::domain_computation::primary_graph) fn matches_settlement(
        &self,
        settled: SettledWorkflowTransition,
    ) -> bool {
        settled.node == self.node
            && settled.occurrence == self.occurrence
            && matches!(
                settled.outcome,
                ApplicationWorkflowControlOutcome::Completed
            )
    }
}

pub(in crate::domain_computation::primary_graph) fn select_terminal_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    settled: &mut [SettledWorkflowTransition],
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let selected = select_current_transition(compiled, instance, settled)?;
    if !matches!(selected.kind(), SelectedWorkflowTransitionKind::Terminal) {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
            selected.node_path(),
        ));
    }
    Ok(selected)
}

pub(in crate::domain_computation::primary_graph) fn select_current_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    settled: &mut [SettledWorkflowTransition],
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let node = select_current_node(compiled, settled)?;
    let kind = match node.kind() {
        CompiledWorkflowNodeKind::Operation {
            operation,
            input_type,
            requires_workflow_authority: true,
        } => SelectedWorkflowTransitionKind::Operation(SelectedWorkflowOperation {
            operation: operation.clone(),
            input_type: input_type.clone(),
        }),
        CompiledWorkflowNodeKind::Assessment {
            query,
            parameter_type,
            result_type,
            binding,
            subject,
        } => SelectedWorkflowTransitionKind::Assessment(SelectedWorkflowAssessment {
            query: query.clone(),
            parameter_type: parameter_type.clone(),
            result_type: result_type.clone(),
            binding: binding.clone(),
            subject: subject.clone(),
        }),
        CompiledWorkflowNodeKind::Condition {
            query,
            parameter_type,
            result_type,
            binding,
        } => SelectedWorkflowTransitionKind::Condition(SelectedWorkflowCondition {
            query: query.clone(),
            parameter_type: parameter_type.clone(),
            result_type: result_type.clone(),
            binding: binding.clone(),
        }),
        CompiledWorkflowNodeKind::Approval {
            capability,
            capability_type,
            operation,
            installed_capability_identity,
        } => SelectedWorkflowTransitionKind::Approval(select_approval(
            compiled,
            node,
            capability,
            capability_type,
            operation,
            installed_capability_identity,
        )?),
        CompiledWorkflowNodeKind::EvidenceJoin { policy } => {
            SelectedWorkflowTransitionKind::EvidenceJoin(*policy)
        }
        CompiledWorkflowNodeKind::Terminal => SelectedWorkflowTransitionKind::Terminal,
        _ => {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                node.path(),
            ))
        }
    };
    let occurrence = u64::try_from(settled.len()).map_err(|_| {
        denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionIdentityUnavailable,
            "workflow transition occurrence exceeds supported identity range",
        )
    })?;
    select_transition(compiled, instance, node, occurrence, kind)
}

pub(in crate::domain_computation::primary_graph) fn select_proposal_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    settled: &mut [SettledWorkflowTransition],
    operation: &str,
    input_type: &str,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let node = select_current_node(compiled, settled)?;
    match node.kind() {
        CompiledWorkflowNodeKind::Operation {
            operation: installed_operation,
            input_type: installed_input_type,
            requires_workflow_authority: false,
        } if installed_operation == operation && installed_input_type == input_type => {}
        _ => {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                node.path(),
            ))
        }
    }
    let occurrence = u64::try_from(settled.len()).map_err(|_| {
        denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionIdentityUnavailable,
            "workflow transition occurrence exceeds supported identity range",
        )
    })?;
    select_transition(
        compiled,
        instance,
        node,
        occurrence,
        SelectedWorkflowTransitionKind::Operation(SelectedWorkflowOperation {
            operation: operation.to_owned(),
            input_type: input_type.to_owned(),
        }),
    )
}

pub(in crate::domain_computation::primary_graph) fn select_proposal_replay_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    settled: SettledWorkflowTransition,
    operation: &str,
    input_type: &str,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let node = compiled
        .nodes()
        .find(|node| node.entity() == settled.node())
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "settled proposal node is absent from the compiled node inventory",
            )
        })?;
    match node.kind() {
        CompiledWorkflowNodeKind::Operation {
            operation: installed_operation,
            input_type: installed_input_type,
            requires_workflow_authority: false,
        } if installed_operation == operation && installed_input_type == input_type => {}
        _ => {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                node.path(),
            ))
        }
    }
    select_transition(
        compiled,
        instance,
        node,
        settled.occurrence(),
        SelectedWorkflowTransitionKind::Operation(SelectedWorkflowOperation {
            operation: operation.to_owned(),
            input_type: input_type.to_owned(),
        }),
    )
}

fn select_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    node: &CompiledWorkflowNode,
    occurrence: u64,
    kind: SelectedWorkflowTransitionKind,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let (identity, identity_bytes) = transition_identity(compiled, instance, node, occurrence)?;
    Ok(SelectedWorkflowTransition {
        node: node.entity(),
        node_path: node.path().to_owned(),
        occurrence,
        identity,
        identity_bytes,
        kind,
    })
}

fn select_current_node<'compiled>(
    compiled: &'compiled CompiledWorkflowDefinition,
    settled: &mut [SettledWorkflowTransition],
) -> Result<&'compiled CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    let head = resolve_head_entity(
        settled,
        compiled.start().entity(),
        |source, outcome, history| {
            unique_successor(compiled, source, outcome, history).map(CompiledWorkflowNode::entity)
        },
    )?;
    compiled
        .nodes()
        .find(|node| node.entity() == head)
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "workflow transition head is absent from the compiled node inventory",
            )
        })
}

fn resolve_head_entity(
    settled: &mut [SettledWorkflowTransition],
    start: EntityId,
    mut successor: impl FnMut(
        EntityId,
        ApplicationWorkflowControlOutcome,
        &[SettledWorkflowTransition],
    ) -> Result<EntityId, WorthQueryApplicationAttemptDenial>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    settled.sort_unstable_by_key(|transition| transition.occurrence);
    let mut expected = start;
    for (index, transition) in settled.iter().enumerate() {
        let expected_occurrence = u64::try_from(index).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionIdentityUnavailable,
                "workflow transition history exceeds supported occurrence range",
            )
        })?;
        if transition.occurrence != expected_occurrence || transition.node != expected {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "workflow transition history is not a contiguous compiled path",
            ));
        }
        expected = successor(transition.node, transition.outcome, &settled[..=index])?;
    }
    Ok(expected)
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}

#[path = "tests.rs"]
#[cfg(test)]
mod tests;
