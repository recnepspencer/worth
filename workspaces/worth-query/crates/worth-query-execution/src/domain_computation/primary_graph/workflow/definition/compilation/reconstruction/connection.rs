use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
};
use worth_relational::facade::identity::EntityId;

use super::{exact_adjacent, observed_u64};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{
        codec::WorkflowConnectionTag,
        compilation::plan::{CompiledWorkflowConnection, CompiledWorkflowConnectionKind},
    },
    schema::WorthQueryWorkflowLayout,
};

pub(super) fn compile_connection(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    nodes: &[EntityId],
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<CompiledWorkflowConnection, WorthQueryApplicationAttemptDenial> {
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: entity,
        kind: layout.connection.entity_kind,
    });
    let family = observed_u64(
        runtime,
        snapshot,
        entity,
        layout.connection.entity_kind,
        &layout.connection.family,
        facts,
    )?;
    let variant = observed_u64(
        runtime,
        snapshot,
        entity,
        layout.connection.entity_kind,
        &layout.connection.variant,
        facts,
    )?;
    let source = exact_adjacent(
        runtime,
        snapshot,
        layout.connection_source_relation,
        entity,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        2,
        facts,
    )?;
    let target = exact_adjacent(
        runtime,
        snapshot,
        layout.connection_target_relation,
        entity,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        2,
        facts,
    )?;
    if !nodes.contains(&source) || !nodes.contains(&target) {
        return Err(invalid_connection());
    }
    let kind = match WorkflowConnectionTag::from_persisted(family, variant) {
        Some(WorkflowConnectionTag::Control(ApplicationWorkflowControlOutcome::Completed)) => {
            CompiledWorkflowConnectionKind::Control(ApplicationWorkflowControlOutcome::Completed)
        }
        Some(WorkflowConnectionTag::Control(ApplicationWorkflowControlOutcome::Approved)) => {
            CompiledWorkflowConnectionKind::Control(ApplicationWorkflowControlOutcome::Approved)
        }
        Some(WorkflowConnectionTag::Control(ApplicationWorkflowControlOutcome::Rejected)) => {
            CompiledWorkflowConnectionKind::Control(ApplicationWorkflowControlOutcome::Rejected)
        }
        Some(WorkflowConnectionTag::Data(ApplicationWorkflowDataFlow::ProposalSubject)) => {
            CompiledWorkflowConnectionKind::Data(ApplicationWorkflowDataFlow::ProposalSubject)
        }
        Some(WorkflowConnectionTag::Data(ApplicationWorkflowDataFlow::AssessmentSubject)) => {
            CompiledWorkflowConnectionKind::Data(ApplicationWorkflowDataFlow::AssessmentSubject)
        }
        Some(WorkflowConnectionTag::Data(ApplicationWorkflowDataFlow::AssessmentEvidence)) => {
            CompiledWorkflowConnectionKind::Data(ApplicationWorkflowDataFlow::AssessmentEvidence)
        }
        Some(WorkflowConnectionTag::Data(ApplicationWorkflowDataFlow::JoinedEvidence)) => {
            CompiledWorkflowConnectionKind::Data(ApplicationWorkflowDataFlow::JoinedEvidence)
        }
        Some(WorkflowConnectionTag::Data(ApplicationWorkflowDataFlow::ApprovalAuthority)) => {
            CompiledWorkflowConnectionKind::Data(ApplicationWorkflowDataFlow::ApprovalAuthority)
        }
        Some(WorkflowConnectionTag::Data(ApplicationWorkflowDataFlow::OperationInput)) => {
            CompiledWorkflowConnectionKind::Data(ApplicationWorkflowDataFlow::OperationInput)
        }
        _ => return Err(invalid_connection()),
    };
    Ok(CompiledWorkflowConnection {
        entity,
        source,
        target,
        kind,
    })
}

fn invalid_connection() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionCompilationUnavailable,
        "published workflow connection shape is invalid",
    )
}
