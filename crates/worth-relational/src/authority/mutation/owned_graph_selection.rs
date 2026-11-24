//! Select the admitted graph in original slots, then derive ordinary mutation effects.
use super::{
    effect_assembly::assemble_effect,
    outcomes::{MutationEvent, MutationOutcome, RecordMutation},
    *,
};
use crate::config::data::MutationConfig;
use crate::identity::data::EntityId;
use crate::identity::data::RelationId;
use crate::identity::data::VersionId;
use crate::mvcc::owned_graph_selection::prepared::PreparedOwnedGraphSelection;
use crate::runtime::state::subsystems::record_identity::PendingRecordAllocations;
use crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings;
use crate::schema::data::AspectContractPlanCatalog;
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::transactions::data::primitives::RecordRef;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
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
        *execution,
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
    let handoff = execution.run_native_with_checkpoint(|context| {
        result = Some((|| {
            let mut outcome = MutationOutcome::with_capacity(execution, 0, 0)?;
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
    });
    let effect = match (handoff, result) {
        (Ok(_), Some(result)) => result?,
        (
            Err(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped,
            )),
            Some(Err(cause)),
        ) => return Err(cause),
        (Err(cause), _) => return Err(request(cause)),
        (Ok(_), None) => {
            unreachable!("completed graph selection callback records its exact result")
        }
    };
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
    crate::publication::ownership::read_request::request(cause.into()).into()
}
fn stale(record: RecordRef, detail: impl Into<String>) -> CommitConflict {
    CommitConflict::new(ConflictClass::MutationStateInconsistency {
        detail: detail.into(),
        evidence: crate::transactions::data::outcomes::conflict_class::MutationStateInconsistencyEvidence::MaterializationTransition { record },
    })
}

fn selection_denial(
    record: RecordRef,
    cause: crate::storage::substrate::MetadataWriteDenial,
) -> MutationApplyDenial {
    match cause {
        crate::storage::substrate::MetadataWriteDenial::Invalid(detail) => {
            stale(record, detail).into()
        }
        crate::storage::substrate::MetadataWriteDenial::Request(cause) => request(cause),
    }
}
