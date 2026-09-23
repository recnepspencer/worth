use worth_relational::facade::identity::EntityId;

use super::{exact_adjacent, observed_optional_text, observed_optional_u64, observed_u64};
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
    let retry_reason = observed_optional_text(
        runtime,
        snapshot,
        entity,
        layout.connection.entity_kind,
        &layout.connection.retry_reason,
        facts,
    )?;
    let retry_maximum_attempts = observed_optional_u64(
        runtime,
        snapshot,
        entity,
        layout.connection.entity_kind,
        &layout.connection.retry_maximum_attempts,
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
    let tag =
        WorkflowConnectionTag::from_persisted(family, variant).ok_or_else(invalid_connection)?;
    let retry_fields_present = retry_reason.is_some() && retry_maximum_attempts.is_some();
    if matches!(tag, WorkflowConnectionTag::Retry(_)) != retry_fields_present {
        return Err(invalid_connection());
    }
    let kind = match tag {
        WorkflowConnectionTag::Control(outcome) => CompiledWorkflowConnectionKind::Control(outcome),
        WorkflowConnectionTag::Data(flow) => CompiledWorkflowConnectionKind::Data(flow),
        WorkflowConnectionTag::Retry(trigger) => {
            let maximum_attempts = retry_maximum_attempts
                .and_then(|value| u16::try_from(value).ok())
                .filter(|value| *value > 0)
                .ok_or_else(invalid_connection)?;
            CompiledWorkflowConnectionKind::Retry {
                trigger,
                reason: retry_reason
                    .filter(|reason| !reason.trim().is_empty())
                    .ok_or_else(invalid_connection)?,
                maximum_attempts,
            }
        }
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
