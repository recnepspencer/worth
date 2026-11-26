//! Select the admitted graph in original slots, then derive ordinary mutation effects.
use super::{
    effect_assembly::assemble_effect,
    outcomes::{MutationEvent, MutationOutcome, RecordMutation},
    *,
};
use crate::contracts::config::data::policies::MutationConfig;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::identity::data::primitives::VersionId;
use crate::mvcc::owned_graph_selection::prepared::PreparedOwnedGraphSelection;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;
use crate::contracts::schema::data::RelationalSchemaRegistry;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use crate::contracts::transactions::data::outcomes::commit_conflict::{CommitConflict, CommitConflictOwnerPort as _};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::RecordRef;
pub mod entities;
pub mod relations;

pub fn apply(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    retained: &PreparedOwnedGraphSelection,
    version: VersionId,
    config: &MutationConfig,
    schema: &RelationalSchemaRegistry,
    plans: &AspectContractPlanCatalog,
    symbols: &mut PendingSymbolMeanings,
    allocations: &mut PendingRecordAllocations,
    budget: Option<crate::contracts::execution::request_work::RequestWorkBudget>,
) -> Result<MutationApplyOutcome, MutationApplyDenial> {
    let graph = &retained.graph;
    // The selected basis lends custody its own preparation admitted.
    let source = graph.entities().selected_basis();
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
                crate::contracts::execution::request_stage::checkpoint(context, 1)
                    .map_err(request)?;
                relations::retire(&mut workspace, id, version, &mut outcome)?;
            }
            // Companion records (ruling (u)'s reversal facts) follow the
            // selection through the same slot transitions and index upkeep.
            for id in graph
                .entities_to_retire()
                .chain(graph.companions().retire.iter().copied())
            {
                crate::contracts::execution::request_stage::checkpoint(context, 1)
                    .map_err(request)?;
                entities::retire(&mut workspace, id, version, &mut outcome)?;
            }
            for &id in graph
                .restored_entities()
                .iter()
                .chain(&graph.companions().restore)
            {
                crate::contracts::execution::request_stage::checkpoint(context, 1)
                    .map_err(request)?;
                entities::select(&mut workspace, source, id, version, &mut outcome, context)?;
            }
            for &id in graph.selected_relations().keys() {
                crate::contracts::execution::request_stage::checkpoint(context, 1)
                    .map_err(request)?;
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
    cause: impl Into<crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> MutationApplyDenial {
    crate::contracts::publication::ownership::read_request::request(cause.into()).into()
}
fn stale(record: RecordRef, detail: impl Into<String>) -> CommitConflict {
    CommitConflict::new(ConflictClass::MutationStateInconsistency {
        detail: detail.into(),
        evidence: crate::contracts::transactions::data::outcomes::conflict_class::mutation_state_inconsistency::MutationStateInconsistencyEvidence::MaterializationTransition { record },
    })
}

fn selection_denial(
    record: RecordRef,
    cause: worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::MetadataWriteDenial,
) -> MutationApplyDenial {
    match cause {
        worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::MetadataWriteDenial::Invalid(detail) => {
            stale(record, detail).into()
        }
        worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::MetadataWriteDenial::Request(cause) => request(cause),
    }
}
