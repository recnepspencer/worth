use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::BranchId;
use crate::mvcc::RelationalTransactionValidationInput;
use crate::runtime::RelationalRuntime;

pub(super) fn owner_options_for_branch(
    restored: &mut RelationalRuntime,
    branch: &BranchId,
) -> Result<RelationalTransactionValidationInput, DurabilityError> {
    let identity = restored.branch_identity(branch).map_err(|denial| {
        DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            format!("recovered branch cannot issue transaction binding: {denial:?}"),
        )
    })?;
    restored
        .transaction_validation_input_for(&identity)
        .map_err(|denial| {
            DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                format!("recovered branch binding was denied: {denial:?}"),
            )
        })
}
