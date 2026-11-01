//! Rebuild actual row DATA from the original sealed mutation outcome inputs.
mod lineage;
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::history::data::CanonicalCommitEnvelope;
use crate::history::OriginalCanonicalInputs;
use crate::identity::data::PartitionId;
use crate::storage::{
    overlay::{PartitionState, WorkingState},
    substrate::SharedMap,
};
use crate::symbols::data::SymbolTextLookup;
use worth_execution::ExecutionRequest;

pub(super) struct AppliedOriginalCommit {
    pub(super) partitions: SharedMap<PartitionId, PartitionState>,
    pub(super) unchanged: bool,
}

pub(super) fn apply_original_commit(
    execution: &ExecutionRequest<'_, '_>,
    partitions: &SharedMap<PartitionId, PartitionState>,
    envelope: &CanonicalCommitEnvelope,
    inputs: &OriginalCanonicalInputs,
    dictionary: &dyn SymbolTextLookup,
) -> Result<AppliedOriginalCommit, Denial> {
    crate::execution::check_request(execution).map_err(Denial::Request)?;
    let mut state = WorkingState::from_original_root_partitions(
        partitions,
        inputs.mutation.adjacency_policy.clone(),
    );
    let mut symbols = crate::runtime::OriginalSymbolMeanings::new(
        dictionary,
        &envelope.symbol_texts,
        &envelope.revision_symbol_references,
    )
    .map_err(|_| Denial::OriginalAuthorityUnavailable)?;
    let mut allocations =
        crate::runtime::OriginalRecordAllocationCursor::new(envelope.record_allocations());
    crate::authority::mutation::apply_original_intents_to_working_state(
        execution,
        &mut state,
        &envelope.merged_plan.merged_intents,
        envelope.commit.version_id,
        &inputs.mutation,
        &inputs.result_schema,
        &mut symbols,
        &mut allocations,
    )
    .map_err(|cause| match cause {
        crate::authority::mutation::MutationApplyDenial::Commit(error) => {
            super::transaction_stop(error)
        }
        crate::authority::mutation::MutationApplyDenial::Symbols(_) => {
            Denial::OriginalAuthorityUnavailable
        }
    })?;
    lineage::install(execution, &mut state, envelope)?;
    let unchanged = state.mutation_journal().is_empty();
    Ok(AppliedOriginalCommit {
        partitions: state.into_original_root_partitions(),
        unchanged,
    })
}
