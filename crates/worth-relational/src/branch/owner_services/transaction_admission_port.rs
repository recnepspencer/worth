use crate::branch::AdmittedRelationalBranchBasis;
use crate::mvcc::{
    BranchBoundRelationalTransaction, RelationalBranchTransactionAdmissionDenial,
    RelationalTransactionIntent,
};

use super::owner_binding::RelationalOwnerServiceBinding;

/// Concrete owner-bound admission service for branch transactions.
#[derive(Debug, Clone)]
pub struct RelationalBranchTransactionAdmissionPort {
    owner: RelationalOwnerServiceBinding,
}

impl RelationalBranchTransactionAdmissionPort {
    pub(super) fn new(owner: RelationalOwnerServiceBinding) -> Self {
        Self { owner }
    }

    pub fn begin_branch_transaction(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        intent: RelationalTransactionIntent,
    ) -> Result<BranchBoundRelationalTransaction, RelationalBranchTransactionAdmissionDenial> {
        let runtime = self
            .owner
            .admitted_runtime()
            .ok_or(RelationalBranchTransactionAdmissionDenial::OwnerUnavailable)?;
        runtime.begin_branch_transaction(basis, intent)
    }
}

#[cfg(test)]
#[path = "transaction_admission_port_tests.rs"]
mod tests;
