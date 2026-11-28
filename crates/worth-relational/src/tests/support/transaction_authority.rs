use super::{BranchId, RelationalRuntime, RelationalTransactionValidationInput};
use crate::mvcc::BranchBoundRelationalTransaction;

/// Observe the configured main identity explicitly and return its owner basis.
pub(crate) fn test_owner_main_basis(
    runtime: &RelationalRuntime,
) -> Result<crate::branch::AdmittedRelationalBranchBasis, crate::branch::RelationalBranchBasisDenial>
{
    let identity = runtime.main_branch_identity();
    runtime.admit_branch_basis(&identity)
}

/// Begin a main-branch transaction after observing its admitted basis.
///
/// Keeping the two borrows inside this helper preserves the same production
/// admission path while allowing fixtures to hold the runtime mutably for the
/// returned transaction.
pub(crate) fn test_owner_begin_transaction_for_main(
    runtime: &RelationalRuntime,
) -> BranchBoundRelationalTransaction {
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("configured main branch remains owner-admissible");
    runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .expect("main basis belongs to its issuing runtime")
}

/// Begin a registered branch transaction after observing its admitted basis.
pub(crate) fn test_owner_begin_transaction_for_branch(
    runtime: &RelationalRuntime,
    branch_id: BranchId,
) -> BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&branch_id)
        .unwrap_or_else(|error| panic!("test branch must be owner-registered: {error:?}"));
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("registered test branch remains owner-admissible");
    runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .expect("branch basis belongs to its issuing runtime")
}

/// Obtain owner-private validation input through the runtime's identity door.
pub(crate) fn test_owner_transaction_validation_input_for_main(
    runtime: &RelationalRuntime,
) -> RelationalTransactionValidationInput {
    let identity = runtime.main_branch_identity();
    runtime
        .transaction_validation_input_for(&identity)
        .expect("configured main branch remains owner-admissible")
}

/// Obtain owner-private validation input for an already registered branch.
pub(crate) fn test_owner_transaction_validation_input_for_branch(
    runtime: &RelationalRuntime,
    branch_id: BranchId,
) -> RelationalTransactionValidationInput {
    let identity = runtime
        .branch_identity(&branch_id)
        .unwrap_or_else(|error| panic!("test branch must be owner-registered: {error:?}"));
    runtime
        .transaction_validation_input_for(&identity)
        .expect("registered test branch remains owner-admissible")
}
