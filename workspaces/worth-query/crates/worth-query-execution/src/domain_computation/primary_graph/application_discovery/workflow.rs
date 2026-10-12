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
    /// The caller cancelled this discovery.
    Cancelled,
    /// The caller's deadline elapsed during discovery.
    DeadlineExceeded,
    /// The caller's bounded execution refused work, memory or execution structure.
    ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind),
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &ApplicationWorkflowDefinitionIdentity,
    ) -> Result<WorthQueryWorkflowDefinitionDiscovery, WorthQueryWorkflowDefinitionDiscoveryDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        execution
            .consult()
            .map_err(|stop| request_denial(stop.into()))?;
        let graph = &self.application().primary_provider.graph;
        let layout = graph.layout.workflow();
        let snapshot = self.application_basis().snapshot_handle();
        let branch = self.product().product_branch();
        graph.with_runtime_mut(execution, |runtime| {
            // A fork that has not committed still needs its own generation of
            // the lineage index before it can be read.
            graph
                .ensure_primary_indexes_for_basis(
                    execution,
                    runtime,
                    self.product().relational_basis(),
                )
                .map_err(|refusal| match refusal {
                    super::super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(cause) => request_denial(cause),
                    super::super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(_)
                    | super::super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(_) => WorthQueryWorkflowDefinitionDiscoveryDenial::IndexUnavailable,
                })?;
            let runtime = &*runtime;
            let lineage = select_lineage(runtime, execution, snapshot, layout, identity.as_str())
                ?.ok_or(WorthQueryWorkflowDefinitionDiscoveryDenial::LineageUnreadable)?;
            let Some(lineage) = lineage else {
                return Ok(WorthQueryWorkflowDefinitionDiscovery::Unpublished);
            };
            let spec = read_definition_field(runtime, execution, snapshot, lineage,
                layout.lineage.entity_kind, &layout.lineage.spec)?
                .ok_or(WorthQueryWorkflowDefinitionDiscoveryDenial::LineageUnreadable)?;
            if spec != text(Spec::IDENTITY.as_str()) {
                return Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ForeignLineage);
            }
            match current_definition(runtime, execution, snapshot, layout, branch, lineage)
                .map_err(|denial| match denial.kind() {
                    super::super::WorthQueryApplicationAttemptDenialKind::Cancelled => {
                        WorthQueryWorkflowDefinitionDiscoveryDenial::Cancelled
                    }
                    super::super::WorthQueryApplicationAttemptDenialKind::DeadlineExceeded => {
                        WorthQueryWorkflowDefinitionDiscoveryDenial::DeadlineExceeded
                    }
                    super::super::WorthQueryApplicationAttemptDenialKind::ExecutionRequest(kind) => WorthQueryWorkflowDefinitionDiscoveryDenial::ExecutionRequest(kind),
                    _ => WorthQueryWorkflowDefinitionDiscoveryDenial::LineageUnreadable,
                })? {
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
    );
    execution.consult().map_err(|stop| {
        super::super::WorthQueryApplicationAttemptDenial::relational_read_stopped(
            stop.into(),
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
fn select_lineage(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    identity: &str,
) -> Result<Option<Option<EntityId>>, WorthQueryWorkflowDefinitionDiscoveryDenial> {
    #[cfg(test)]
    let _scope = super::super::application_attempt::consult_observation::ReadConsultScopeForTest::enter_for_test(
        super::super::application_attempt::consult_observation::Door::Discovery);
    let selection = observe_indexed_entity_selection(
        runtime,
        execution,
        snapshot,
        layout.lineage.identity_index_id,
        layout.lineage.entity_kind,
        layout.lineage.identity.clone(),
        text(identity),
        LINEAGE_LOOKUP_LIMIT,
    )
    .map_err(|refusal| match refusal {
        super::super::application_attempt::WorthQueryIndexedSelectionRefusal::Execution(cause) => {
            request_denial(cause)
        }
        super::super::application_attempt::WorthQueryIndexedSelectionRefusal::Overflowed
        | super::super::application_attempt::WorthQueryIndexedSelectionRefusal::Unavailable => {
            WorthQueryWorkflowDefinitionDiscoveryDenial::LineageUnreadable
        }
    })?;
    let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } = selection
    else {
        return Ok(None);
    };
    Ok(match candidates.as_slice() {
        [] => Some(None),
        [lineage] => Some(Some(*lineage)),
        _ => None,
    })
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}

fn request_denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> WorthQueryWorkflowDefinitionDiscoveryDenial {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
    match super::super::provider::relational_execution_denial::relational_execution_kind(
        cause, None,
    ) {
        Ok(kind) => WorthQueryWorkflowDefinitionDiscoveryDenial::ExecutionRequest(kind),
        Err(Stop::Cancelled) => WorthQueryWorkflowDefinitionDiscoveryDenial::Cancelled,
        Err(Stop::TimedOut) => WorthQueryWorkflowDefinitionDiscoveryDenial::DeadlineExceeded,
    }
}
fn read_definition_field(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> Result<Option<AspectValue>, WorthQueryWorkflowDefinitionDiscoveryDenial> {
    execution
        .consult()
        .map_err(|stop| request_denial(stop.into()))?;
    let value = observe_field_value(runtime, execution, snapshot, entity, kind, locator);
    execution
        .consult()
        .map_err(|stop| request_denial(stop.into()))?;
    Ok(value)
}
#[cfg(test)]
mod request_stops;
