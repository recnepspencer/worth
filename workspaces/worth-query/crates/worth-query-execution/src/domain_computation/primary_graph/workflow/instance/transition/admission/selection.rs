use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
};
use crate::domain_computation::primary_graph::workflow::instance::{
    SettledWorkflowTransition, WorkflowInstanceProgress, WorkflowTransitionProgressBasis,
    WorkflowTransitionReplayProjection,
};

mod approval;
mod collection;
mod identity;
mod replay;
use approval::select_approval;
pub(in crate::domain_computation::primary_graph) use approval::SelectedWorkflowApproval;
pub(in crate::domain_computation::primary_graph) use collection::select_assessment_collection;
use identity::transition_identity;
pub(in crate::domain_computation::primary_graph) use replay::select_settled_replay_transition;
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowTransition {
    pub(super) node: EntityId,
    pub(super) node_path: String,
    pub(super) occurrence: u64,
    pub(super) identity: String,
    pub(super) identity_bytes: [u8; 32],
    pub(super) progress_basis: Option<WorkflowTransitionProgressBasis>,
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
    NavigationBack,
    Terminal,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct SelectedWorkflowOperation {
    pub(in crate::domain_computation::primary_graph) operation: String,
    pub(in crate::domain_computation::primary_graph) input_type: String,
    pub(in crate::domain_computation::primary_graph) binding: Option<String>,
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

    pub(in crate::domain_computation::primary_graph) const fn identity_bytes(&self) -> &[u8; 32] {
        &self.identity_bytes
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
        settled.node() == self.node
            && settled.occurrence() == self.occurrence
            && matches!(
                settled.outcome(),
                ApplicationWorkflowControlOutcome::Completed
            )
    }
}

pub(in crate::domain_computation::primary_graph) fn select_terminal_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    progress_basis: &WorkflowTransitionProgressBasis,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let selected = select_current_transition(compiled, instance, progress_basis)?;
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
    progress_basis: &WorkflowTransitionProgressBasis,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let progress = progress_basis.progress();
    let node = select_current_node(compiled, progress)?;
    let kind = match node.kind() {
        CompiledWorkflowNodeKind::Operation {
            operation,
            input_type,
            binding,
            requires_workflow_authority: true,
        } => SelectedWorkflowTransitionKind::Operation(SelectedWorkflowOperation {
            operation: operation.clone(),
            input_type: input_type.clone(),
            binding: binding.clone(),
        }),
        CompiledWorkflowNodeKind::Assessment {
            query,
            parameter_type,
            result_type,
            binding,
            subject,
            ..
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
    let occurrence = progress.next_occurrence();
    select_transition(
        compiled,
        instance,
        node,
        occurrence,
        kind,
        Some(progress_basis.clone()),
    )
}

pub(in crate::domain_computation::primary_graph) fn select_navigation_back_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    progress_basis: &WorkflowTransitionProgressBasis,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let progress = progress_basis.progress();
    progress.back_target()?;
    let node = select_current_node(compiled, progress)?;
    if matches!(node.kind(), CompiledWorkflowNodeKind::Terminal) {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled,
            node.path(),
        ));
    }
    let occurrence = progress.next_occurrence();
    let (identity, identity_bytes) = transition_identity(
        compiled,
        instance,
        node,
        occurrence,
        progress.back_edge_iterations(),
        true,
    )?;
    Ok(SelectedWorkflowTransition {
        node: node.entity(),
        node_path: node.path().to_owned(),
        occurrence,
        identity,
        identity_bytes,
        progress_basis: Some(progress_basis.clone()),
        kind: SelectedWorkflowTransitionKind::NavigationBack,
    })
}

pub(in crate::domain_computation::primary_graph) fn select_proposal_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    progress_basis: &WorkflowTransitionProgressBasis,
    operation: &str,
    input_type: &str,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let progress = progress_basis.progress();
    let node = select_current_node(compiled, progress)?;
    match node.kind() {
        CompiledWorkflowNodeKind::Operation {
            operation: installed_operation,
            input_type: installed_input_type,
            requires_workflow_authority: false,
            ..
        } if installed_operation == operation && installed_input_type == input_type => {}
        _ => {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                node.path(),
            ))
        }
    }
    let occurrence = progress.next_occurrence();
    select_transition(
        compiled,
        instance,
        node,
        occurrence,
        SelectedWorkflowTransitionKind::Operation(SelectedWorkflowOperation {
            operation: operation.to_owned(),
            input_type: input_type.to_owned(),
            binding: None,
        }),
        Some(progress_basis.clone()),
    )
}

pub(in crate::domain_computation::primary_graph) fn select_proposal_replay_transition(
    compiled: &CompiledWorkflowDefinition,
    settled: SettledWorkflowTransition,
    operation: &str,
    input_type: &str,
    replay: &WorkflowTransitionReplayProjection,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let node = compiled.node(settled.node()).ok_or_else(|| {
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
            ..
        } if installed_operation == operation && installed_input_type == input_type => {}
        _ => {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                node.path(),
            ))
        }
    }
    if replay.node_path != node.path() || replay.terminal {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "proposal replay does not match its settled node",
        ));
    }
    Ok(SelectedWorkflowTransition {
        node: node.entity(),
        node_path: node.path().to_owned(),
        occurrence: settled.occurrence(),
        identity: replay.identity.clone(),
        identity_bytes: replay.identity_bytes,
        progress_basis: None,
        kind: SelectedWorkflowTransitionKind::Operation(SelectedWorkflowOperation {
            operation: operation.to_owned(),
            input_type: input_type.to_owned(),
            binding: None,
        }),
    })
}

fn select_transition(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    node: &CompiledWorkflowNode,
    occurrence: u64,
    kind: SelectedWorkflowTransitionKind,
    progress_basis: Option<WorkflowTransitionProgressBasis>,
) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let iterations = progress_basis
        .as_ref()
        .map(|basis| basis.progress().back_edge_iterations());
    let empty = im::OrdMap::new();
    let (identity, identity_bytes) = transition_identity(
        compiled,
        instance,
        node,
        occurrence,
        iterations.unwrap_or(&empty),
        false,
    )?;
    Ok(SelectedWorkflowTransition {
        node: node.entity(),
        node_path: node.path().to_owned(),
        occurrence,
        identity,
        identity_bytes,
        progress_basis,
        kind,
    })
}

fn select_current_node<'compiled>(
    compiled: &'compiled CompiledWorkflowDefinition,
    progress: &WorkflowInstanceProgress,
) -> Result<&'compiled CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    compiled.node(progress.head()).ok_or_else(|| {
        denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "workflow transition head is absent from the compiled node inventory",
        )
    })
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
