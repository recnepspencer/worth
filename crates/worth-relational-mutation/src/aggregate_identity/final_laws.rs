//! Metadata-dependent laws consume the final real state, including no-op consequences.
use super::funding;
use super::preparation::derived_laws::{validate, LawStage};
use crate::authority::mutation::MutationEffect;
use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::validation::engine::state_view::InvariantStateView;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState;
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub fn validate_final_derived_laws(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::contracts::execution::request_work::RequestWorkBudget>,
    authority: &RelationalBranchRootSchemaAuthority,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    effect: &MutationEffect,
    version: VersionId,
    symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
) -> Result<super::AggregateReadObservations, TransactionCommitError> {
    if (*authority.registry().owner_derived_relations()).is_empty()
        || (*authority.registry().owner_native_private_kinds()).is_empty()
    {
        return Ok(super::AggregateReadObservations::default());
    }
    worth_relational_storage::internal_port::storage::residency::head_probe::probe_head(
        execution,
        &selected.with_symbol_admission(symbols),
        |partitions| {
            let selected = worth_relational_branch::internal_port::branch::selected_read::SelectedRelationalBranchRead::in_round(
                selected, partitions,
            );
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
                    effect.publication.has_entity_deletions,
                    LawStage::FinalMetadata,
                    work,
                )?;
                Ok(LawsValidated)
            })
            .map(|(_, _, reads)| reads)
        },
    )
    .map_err(|failure| {
        worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(basis, failure)
    })?
}
struct LawsValidated;
impl worth_execution::ChargedBytes for LawsValidated {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
