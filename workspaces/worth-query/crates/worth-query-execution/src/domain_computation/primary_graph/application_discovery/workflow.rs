//! Governed discovery of the workflow definitions a selected branch holds.
//! It reads one admitted occurrence and mints non-authoritative references:
//! a discovered definition grants no start, and every start re-checks it
//! against its own branch's truth.

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowDefinitionContentIdentity, ApplicationWorkflowDefinitionIdentity,
    ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::super::application_attempt::{
    observe_adjacency, observe_field_value, observe_indexed_entity_selection,
    PublishedWorkflowDefinitionRef, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationObservedFact,
};
use super::super::product_operation::WorthQuerySelectedProductOperation;
use super::super::workflow::schema::WorthQueryWorkflowLayout;

/// A lineage is unique per definition identity, so a second candidate is
/// already a corrupt branch.
const LINEAGE_LOOKUP_LIMIT: usize = 2;
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
    /// The identity names a lineage another workflow spec published. Naming
    /// the spec that published it is the only way to discover it.
    ForeignLineage,
    /// The lineage's records cannot be read within their fixed bounds, so
    /// no retry of the same discovery can succeed.
    LineageUnreadable,
    /// The lineage index could not be made current for the selected
    /// occurrence within its reconstruction budget.
    IndexUnavailable,
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// The definition this selected occurrence holds current for `identity`.
    /// The answer is exact for this occurrence only; a later start that finds
    /// its definition superseded or retired says so with a typed outcome.
    pub fn discover_workflow_definition<Spec>(
        &self,
        identity: &ApplicationWorkflowDefinitionIdentity,
    ) -> Result<WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowDefinitionDiscoveryDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let graph = &self.application().primary_provider.graph;
        let layout = graph.layout.workflow();
        let snapshot = self.application_basis().snapshot_handle();
        let branch = self.product().product_branch();
        graph.with_runtime_mut(|runtime| {
            // A fork that has not committed still needs its own generation of
            // the lineage index before it can be read.
            graph
                .ensure_primary_indexes_for_basis(runtime, self.product().relational_basis())
                .map_err(|_| WorthQueryWorkflowDefinitionDiscoveryDenial::IndexUnavailable)?;
            let runtime = &*runtime;
            let lineage = select_lineage(runtime, snapshot, layout, identity.as_str())
                .ok_or(WorthQueryWorkflowDefinitionDiscoveryDenial::LineageUnreadable)?;
            let Some(lineage) = lineage else {
                return Ok(WorthQueryWorkflowDefinitionDiscovery::Unpublished);
            };
            let spec = observe_field_value(
                runtime,
                snapshot,
                lineage,
                layout.lineage.entity_kind,
                &layout.lineage.spec,
            )
            .ok_or(WorthQueryWorkflowDefinitionDiscoveryDenial::LineageUnreadable)?;
            if spec != text(Spec::IDENTITY.as_str()) {
                return Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ForeignLineage);
            }
            match current_definition(runtime, snapshot, layout, branch, lineage) {
                Some(WorkflowDefinitionCurrentness::Current(current)) => {
                    Ok(WorthQueryWorkflowDefinitionDiscovery::Current(current))
                }
                Some(WorkflowDefinitionCurrentness::Retired) => {
                    Ok(WorthQueryWorkflowDefinitionDiscovery::Retired)
                }
                None => Err(WorthQueryWorkflowDefinitionDiscoveryDenial::LineageUnreadable),
            }
        })
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
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    branch: crate::basis::WorthQueryProductBranch,
    lineage: EntityId,
) -> Option<WorkflowDefinitionCurrentness> {
    let relations = observe_adjacency(
        runtime,
        snapshot,
        layout.current_definition_relation,
        lineage,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        CURRENT_DEFINITION_LIMIT,
    )?;
    let current = match relations.as_slice() {
        [] => return Some(WorkflowDefinitionCurrentness::Retired),
        [current] => current.to,
        _ => return None,
    };
    let AspectValue::String(InternedString::Raw(recorded)) = observe_field_value(
        runtime,
        snapshot,
        current,
        layout.definition.entity_kind,
        &layout.definition.content_identity,
    )?
    else {
        return None;
    };
    let content_identity = ApplicationWorkflowDefinitionContentIdentity::from_recorded(&recorded)?;
    Some(WorkflowDefinitionCurrentness::Current(
        PublishedWorkflowDefinitionRef::retained(branch, current, content_identity),
    ))
}

/// The lineage published under `identity`, `Some(None)` when there is none.
fn select_lineage(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    identity: &str,
) -> Option<Option<EntityId>> {
    let selection = observe_indexed_entity_selection(
        runtime,
        snapshot,
        layout.lineage.identity_index_id,
        layout.lineage.entity_kind,
        layout.lineage.identity.clone(),
        text(identity),
        LINEAGE_LOOKUP_LIMIT,
    )?;
    let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } = selection
    else {
        return None;
    };
    match candidates.as_slice() {
        [] => Some(None),
        [lineage] => Some(Some(*lineage)),
        _ => None,
    }
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}
