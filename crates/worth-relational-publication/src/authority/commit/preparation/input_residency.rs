//! Admit the declared input footprint before a draft can issue identities.
use std::collections::BTreeSet;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_contracts::internal_port as contracts;

use contracts::identity::data::primitives::PartitionId;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

/// An input partition is known independently of provisional mutation results.
/// Demand it before identity issuance; genuinely discovered reads still use
/// the enclosing probe and abandon any identities they have already spent.
pub fn demand_mutation_inputs<'intent>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    symbols: &RuntimeSymbolTable,
    intents: impl IntoIterator<Item = &'intent MutationIntent>,
    mut partitions: BTreeSet<PartitionId>,
) -> Result<(), TransactionCommitError> {
    // Preview, validation or draft admission already consulted its named stage.
    // This declared demand is not another candidate round. Every actual load
    // consults the original request at the residency boundary.
    for intent in intents {
        intent.seed_touched_partitions(&mut partitions);
    }
    basis
        .owner_inner()
        .root
        .demand_regions_head(execution, symbols, partitions)
        .map_err(|failure| worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(basis, failure))
}
