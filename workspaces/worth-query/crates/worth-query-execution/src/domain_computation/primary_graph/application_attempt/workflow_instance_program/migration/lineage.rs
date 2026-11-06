//! The one successor a migrated source links from.

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use crate::domain_computation::graph_layout::workflow::WorthQueryWorkflowLayout;
use {
    super::super::super::observe_adjacency, super::super::super::observe_field_value,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationAdjacencyDirection,
};

/// The identity of the one successor a migrated source links from.
pub fn successor_identity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    source: EntityId,
) -> Result<Option<String>, super::super::super::WorthQueryApplicationAttemptDenial> {
    let successors = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.instance_migrated_from_relation,
        source,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        2,
    )?;
    let Some(successors) = successors else {
        return Ok(None);
    };
    let [successor] = successors.as_slice() else {
        return Ok(None);
    };
    Ok(
        match observe_field_value(
            runtime,
            execution,
            snapshot,
            successor.from,
            layout.instance.entity_kind,
            &layout.instance.identity,
        )? {
            Some(AspectValue::String(InternedString::Raw(identity))) => Some(identity),
            _ => None,
        },
    )
}
