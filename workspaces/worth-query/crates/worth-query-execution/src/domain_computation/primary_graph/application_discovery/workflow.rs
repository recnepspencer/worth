//! Governed discovery of the workflow definitions a selected branch holds.
//! It reads one admitted occurrence and mints non-authoritative references:
//! a discovered definition grants no start, and every start re-checks it
//! against its own branch's truth.

mod admission;
pub(in crate::domain_computation::primary_graph) use admission::{
    admit as admit_workflow_read, WorthQueryWorkflowReadDenial as WorkflowReadAdmissionDenial,
};
mod audit;
mod definition;
mod enumeration;
pub(in crate::domain_computation::primary_graph) mod indexes;
mod original_intent;
mod read;
mod selection;
pub use definition::WorthQueryWorkflowDefinitionRead;
pub use enumeration::*;
mod text;
mod view;
pub use admission::WorthQueryWorkflowReadDenial;
pub use view::*;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity;
use worth_relational::facade::identity::EntityId;

use super::super::application_attempt::{
    observe_adjacency, observe_field_value, PublishedWorkflowDefinitionRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationObservedFact,
};
use super::super::workflow::schema::WorthQueryWorkflowLayout;

/// A lineage is unique per definition identity, so a second candidate is
/// already a corrupt branch.
/// A lineage holds at most one current definition.
const CURRENT_DEFINITION_LIMIT: usize = 2;

/// What one branch holds current under a workflow definition identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowDefinitionDiscovery {
    /// The definition a new start on this branch names.
    Current(PublishedWorkflowDefinitionRef),
    /// The lineage was retired. Its pinned instances still run, and no
    /// definition is current until one is published again.
    Retired,
    /// No definition was ever published under this identity on the branch.
    Unpublished,
}

/// What a lineage holds current on the snapshot read.
pub(in crate::domain_computation::primary_graph) enum WorkflowDefinitionCurrentness {
    Current(PublishedWorkflowDefinitionRef),
    Retired,
}

/// The definition `lineage` holds current, named as `branch` holds it, or
/// `None` when the lineage's current relation or the definition's recorded
/// content identity cannot be read.
pub(in crate::domain_computation::primary_graph) fn current_definition(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    branch: crate::basis::WorthQueryProductBranch,
    lineage: EntityId,
) -> Result<Option<WorkflowDefinitionCurrentness>, super::super::WorthQueryApplicationAttemptDenial>
{
    let relations = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.current_definition_relation,
        lineage,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        CURRENT_DEFINITION_LIMIT,
    )?;
    let Some(relations) = relations else {
        return Ok(None);
    };
    let current = match relations.as_slice() {
        [] => return Ok(Some(WorkflowDefinitionCurrentness::Retired)),
        [current] => current.to,
        _ => return Ok(None),
    };
    let recorded = observe_field_value(
        runtime,
        execution,
        snapshot,
        current,
        layout.definition.entity_kind,
        &layout.definition.content_identity,
    )?;
    execution.check_live().map_err(|stop| {
        super::super::WorthQueryApplicationAttemptDenial::request_read_stopped(
            stop,
            "workflow definition discovery",
        )
    })?;
    let Some(AspectValue::String(InternedString::Raw(recorded))) = recorded else {
        return Ok(None);
    };
    let Some(content_identity) =
        ApplicationWorkflowDefinitionContentIdentity::from_recorded(&recorded)
    else {
        return Ok(None);
    };
    Ok(Some(WorkflowDefinitionCurrentness::Current(
        PublishedWorkflowDefinitionRef::retained(branch, current, content_identity),
    )))
}
