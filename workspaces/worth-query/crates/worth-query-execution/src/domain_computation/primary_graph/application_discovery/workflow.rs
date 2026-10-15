//! Governed discovery of the workflow definitions a selected branch holds.
//! It reads one admitted occurrence and mints non-authoritative references:
//! a discovered definition grants no start, and every start re-checks it
//! against its own branch's truth.

mod admission;
pub(in crate::domain_computation::primary_graph) use admission::{
    admit as admit_workflow_read, WorthQueryWorkflowReadDenial as WorkflowReadAdmissionDenial,
};
mod audit;
mod original_intent;
mod read;
mod view;
pub use admission::WorthQueryWorkflowReadDenial;
pub use view::*;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowDefinitionContentIdentity, ApplicationWorkflowDefinitionIdentity,
    ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::super::application_attempt::{
    observe_adjacency, observe_field_value, PublishedWorkflowDefinitionRef,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationObservedFact,
};
use super::super::product_operation::WorthQuerySelectedProductOperation;
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

/// Why a branch's workflow definitions could not be discovered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowDefinitionDiscoveryDenial {
    /// A native request stopped while reading the selected occurrence.
    ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind),
    /// The identity names a lineage another workflow spec published. Naming
    /// the spec that published it is the only way to discover it.
    ForeignLineage,
    /// The caller cancelled this discovery.
    Cancelled,
    /// The caller's deadline elapsed during discovery.
    DeadlineExceeded,
    /// The caller's bounded execution refused work, memory or execution structure.
    /// The lineage's records cannot be read within their fixed bounds, so
    /// no retry of the same discovery can succeed.
    LineageUnreadable,
    /// The lineage index could not be made current for the selected
    /// occurrence within its reconstruction budget.
    IndexUnavailable,
    ObservationDisabled,
    ObservationNotImplemented,
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// The definition this selected occurrence holds current for `identity`.
    /// The answer is exact for this occurrence only; a later start that finds
    /// its definition superseded or retired says so with a typed outcome.
    pub fn discover_workflow_definition<Spec>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        _identity: &ApplicationWorkflowDefinitionIdentity,
    ) -> Result<WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowDefinitionDiscoveryDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        execution.consult().map_err(|stop| {
            discovery_execution_denial(crate::facade::primary_graph::relational_execution_kind(
                stop.into(),
                None,
            ))
        })?;
        match Spec::observation_policy() {
            worth_query_declaration::facade::application_program::ApplicationWorkflowObservationPolicy::Disabled =>
                Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled),
            worth_query_declaration::facade::application_program::ApplicationWorkflowObservationPolicy::Governed { .. } =>
                Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationNotImplemented),
        }
    }
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

/// The lineage published under `identity`, `Some(None)` when there is none.
fn discovery_execution_denial(
    cause: Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
) -> WorthQueryWorkflowDefinitionDiscoveryDenial {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    match cause {
        Ok(kind) => WorthQueryWorkflowDefinitionDiscoveryDenial::ExecutionRequest(kind),
        Err(Control::Cancelled) => WorthQueryWorkflowDefinitionDiscoveryDenial::Cancelled,
        Err(Control::TimedOut) => WorthQueryWorkflowDefinitionDiscoveryDenial::DeadlineExceeded,
    }
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}

#[cfg(test)]
mod request_stops;
