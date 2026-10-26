//! Admit the declared input footprint before a draft can issue identities.
use std::collections::BTreeSet;

use crate::branch::AdmittedRelationalBranchBasis;
use crate::identity::data::PartitionId;
use crate::runtime::RuntimeSymbolTable;
use crate::storage::residency::RelationalResidencyDemandFailure;
use crate::transactions::data::{MutationIntent, TransactionCommitError};

/// An input partition is known independently of provisional mutation results.
/// Demand it before identity issuance; genuinely discovered reads still use
/// the enclosing probe and abandon any identities they have already spent.
pub(crate) fn demand_mutation_inputs<'intent>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    symbols: &RuntimeSymbolTable,
    intents: impl IntoIterator<Item = &'intent MutationIntent>,
    mut partitions: BTreeSet<PartitionId>,
) -> Result<(), TransactionCommitError> {
    crate::execution::check_request(execution).map_err(|cause| {
        super::storage_read_denial(basis, RelationalResidencyDemandFailure::Request(cause))
    })?;
    for intent in intents {
        intent.seed_touched_partitions(&mut partitions);
    }
    basis
        .inner
        .root
        .demand_regions_full(execution, symbols, partitions)
        .map_err(|failure| super::storage_read_denial(basis, failure))
}
