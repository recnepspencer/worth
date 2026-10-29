//! A discarded current invariant round preserves typed faults and side effects.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

impl PreparationDoor {
    const ALL: [Self; 3] = [Self::Validation, Self::Candidate, Self::Commit];
    fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<(), RelationalStorageUnavailable> {
        let transaction = transaction(execution, runtime);
        match self {
            Self::Validation => transaction.validate(execution, runtime).map(|_| ()),
            Self::Candidate => runtime
                .prepare_branch_transaction(execution, transaction)
                .map(|_| ()),
            Self::Commit => transaction.commit(execution, runtime).map(|_| ()),
        }
        .map_err(storage)
    }
}

#[cfg(test)]
#[path = "cold_preparation_failure_matrix/exhaustive.rs"]
mod exhaustive;
