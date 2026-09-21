use sha2::{Digest, Sha256};
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::EntityId;

use super::SettledWorkflowTransition;
use crate::domain_computation::canonical_operation_material;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
};

mod approval;
mod replay;
use approval::select_approval;
pub(in crate::domain_computation::primary_graph) use approval::SelectedWorkflowApproval;
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
    Operation,
    Assessment(SelectedWorkflowAssessment),
    Approval(SelectedWorkflowApproval),
    EvidenceJoin,
    Terminal,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowAssessment {
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
        CompiledWorkflowNodeKind::Assessment {
            query,
            parameter_type,
            result_type,
            binding,
        } => SelectedWorkflowTransitionKind::Assessment(SelectedWorkflowAssessment {
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
        CompiledWorkflowNodeKind::EvidenceJoin => SelectedWorkflowTransitionKind::EvidenceJoin,
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
        SelectedWorkflowTransitionKind::Operation,
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
        SelectedWorkflowTransitionKind::Operation,
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

fn transition_identity(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    node: &CompiledWorkflowNode,
    occurrence: u64,
) -> Result<(String, [u8; 32]), WorthQueryApplicationAttemptDenial> {
    let material = canonical_operation_material(vec![
        (
            "workflow.instance.partition",
            instance.partition_value().to_string(),
        ),
        (
            "workflow.instance.slot",
            instance.local_slot_value().to_string(),
        ),
        (
            "workflow.instance.generation",
            instance.generation_value().to_string(),
        ),
        (
            "workflow.definition.partition",
            compiled.definition().partition_value().to_string(),
        ),
        (
            "workflow.definition.slot",
            compiled.definition().local_slot_value().to_string(),
        ),
        (
            "workflow.definition.generation",
            compiled.definition().generation_value().to_string(),
        ),
        (
            "workflow.node.partition",
            node.entity().partition_value().to_string(),
        ),
        (
            "workflow.node.slot",
            node.entity().local_slot_value().to_string(),
        ),
        (
            "workflow.node.generation",
            node.entity().generation_value().to_string(),
        ),
        ("workflow.occurrence", occurrence.to_string()),
    ]);
    let identity_bytes: [u8; 32] = Sha256::digest(material.as_bytes()).into();
    let identity = encode(identity_bytes)?;
    Ok((identity, identity_bytes))
}

fn select_current_node<'compiled>(
    compiled: &'compiled CompiledWorkflowDefinition,
    settled: &mut [SettledWorkflowTransition],
) -> Result<&'compiled CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    let head = resolve_head_entity(settled, compiled.start().entity(), |source, outcome| {
        unique_successor(compiled, source, outcome).map(CompiledWorkflowNode::entity)
    })?;
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
        expected = successor(transition.node, transition.outcome)?;
    }
    Ok(expected)
}

fn unique_successor<'compiled>(
    compiled: &'compiled CompiledWorkflowDefinition,
    source: EntityId,
    outcome: ApplicationWorkflowControlOutcome,
) -> Result<&'compiled CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    let target = unique_successor_entity(
        compiled
            .control_successors(source, outcome)
            .map(CompiledWorkflowNode::entity),
    )?;
    compiled
        .nodes()
        .find(|node| node.entity() == target)
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "settled workflow successor is absent from the compiled node inventory",
            )
        })
}

fn unique_successor_entity(
    targets: impl IntoIterator<Item = EntityId>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let mut targets = targets.into_iter();
    let Some(target) = targets.next() else {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "settled workflow transition has no compiled successor",
        ));
    };
    if targets.next().is_some() {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "settled workflow transition has ambiguous compiled successors",
        ));
    }
    Ok(target)
}

fn encode(identity: [u8; 32]) -> Result<String, WorthQueryApplicationAttemptDenial> {
    let mut text = String::with_capacity(64);
    for byte in identity {
        use std::fmt::Write;
        write!(&mut text, "{byte:02x}").map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionIdentityUnavailable,
                "workflow transition identity encoding failed",
            )
        })?;
    }
    Ok(text)
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
