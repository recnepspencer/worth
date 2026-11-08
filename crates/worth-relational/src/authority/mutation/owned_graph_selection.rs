//! Select the admitted graph in original slots, then derive ordinary mutation effects.
use super::{
    effect_assembly::assemble_effect,
    outcomes::{MutationEvent, MutationOutcome, RecordMutation},
    *,
};
use crate::{
    config::data::MutationConfig,
    identity::data::{EntityId, RelationId, VersionId},
    mvcc::owned_graph_selection::PreparedOwnedGraphSelection,
    runtime::{PendingRecordAllocations, PendingSymbolMeanings},
    schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry},
    storage::overlay::{CurrentPartitionAccess, WorkingState},
    transactions::data::{CommitConflict, ConflictClass, RecordRef},
};
mod entities;
mod relations;

pub(crate) fn apply(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    retained: &PreparedOwnedGraphSelection,
    version: VersionId,
    config: &MutationConfig,
    schema: &RelationalSchemaRegistry,
    plans: &AspectContractPlanCatalog,
    symbols: &mut PendingSymbolMeanings,
    allocations: &mut PendingRecordAllocations,
    budget: Option<crate::execution::RequestWorkBudget>,
) -> Result<MutationApplyOutcome, MutationApplyDenial> {
    let graph = &retained.graph;
    let source = graph.entities().selected_basis().inner.root.as_ref();
    let mut workspace = MutationWorkspace::new(
        state,
        symbols,
        config,
        schema,
        plans,
        version,
        BranchLocalDeleteAllowance::default(),
        Some(allocations),
        budget,
    );
    let mut result = None;
    execution
        .run_native_with_checkpoint(|context| {
            result = Some((|| {
                let mut outcome = MutationOutcome::with_capacity(0, 0);
                for id in graph.relations_to_retire() {
                    crate::execution::request_stage::checkpoint(context, 1).map_err(request)?;
                    relations::retire(&mut workspace, id, version, &mut outcome)?;
                }
                for id in graph.entities_to_retire() {
                    crate::execution::request_stage::checkpoint(context, 1).map_err(request)?;
                    entities::retire(&mut workspace, id, version, &mut outcome)?;
                }
                for &id in graph.restored_entities() {
                    crate::execution::request_stage::checkpoint(context, 1).map_err(request)?;
                    entities::select(&mut workspace, source, id, version, &mut outcome, context)?;
                }
                for &id in graph.selected_relations().keys() {
                    crate::execution::request_stage::checkpoint(context, 1).map_err(request)?;
                    relations::select(&mut workspace, source, id, version, &mut outcome, context)?;
                }
                assemble_effect(outcome, &mut workspace)
            })());
        })
        .map_err(request)?;
    let effect = result.expect("completed graph selection apply")?;
    let preparation_telemetry = workspace.preparation_telemetry();
    let (created_entities, created_relations) = workspace.into_created_bindings();
    Ok(MutationApplyOutcome {
        effect,
        preparation_telemetry,
        created_entities,
        created_relations,
    })
}
fn request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> MutationApplyDenial {
    crate::publication::ownership::request(cause.into()).into()
}
fn stale(record: RecordRef, detail: impl Into<String>) -> CommitConflict {
    CommitConflict::new(ConflictClass::MutationStateInconsistency {
        detail: detail.into(),
        evidence: crate::transactions::data::MutationStateInconsistencyEvidence::MaterializationTransition { record },
    })
}
