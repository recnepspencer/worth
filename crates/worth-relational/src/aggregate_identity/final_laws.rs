//! Metadata-dependent laws consume the final real state, including no-op consequences.
use super::funding;
use super::preparation::derived_laws::{validate, LawStage};
use crate::authority::mutation::MutationEffect;
use crate::branch::{RelationalBranchRootSchemaAuthority, SelectedRelationalBranchState};
use crate::identity::data::VersionId;
use crate::storage::overlay::{OverlayStateView, WorkingState};
use crate::transactions::data::TransactionCommitError;
use crate::validation::engine::state_view::InvariantStateView;

pub(crate) fn validate_final_derived_laws(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::execution::RequestWorkBudget>,
    authority: &RelationalBranchRootSchemaAuthority,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    effect: &MutationEffect,
    version: VersionId,
    symbols: &crate::runtime::RuntimeSymbolTable,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
) -> Result<super::AggregateReadObservations, TransactionCommitError> {
    if authority.registry().derived_relations.is_empty()
        || authority.registry().aggregate_private_kinds.is_empty()
    {
        return Ok(super::AggregateReadObservations::default());
    }
    crate::storage::residency::probe_head(
        execution,
        &selected.with_symbol_admission(symbols),
        |partitions| {
            let selected =
                crate::branch::SelectedRelationalBranchRead::in_round(selected, partitions);
            funding::run_observed(execution, budget, |work| {
                work.poll()?;
                let before = InvariantStateView::new(&selected, selected.version_id());
                let overlay = OverlayStateView::new(&selected, working);
                let after = InvariantStateView::new(&overlay, version);
                validate(
                    authority.registry(),
                    &before,
                    &after,
                    &effect.adjacency.deltas,
                    &effect.publication.changed_records,
                    LawStage::FinalMetadata,
                    work,
                )?;
                Ok(LawsValidated)
            })
            .map(|(_, _, reads)| reads)
        },
    )
    .map_err(|failure| crate::authority::commit::preparation::storage_read_denial(basis, failure))?
}
struct LawsValidated;
impl worth_execution::ChargedBytes for LawsValidated {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
