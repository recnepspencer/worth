use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::runtime::RelationalRuntime;

pub(super) fn recovered_branch_basis(
    restored: &mut RelationalRuntime,
    branch_id: &crate::history::data::BranchId,
) -> Result<crate::branch::AdmittedRelationalBranchBasis, DurabilityError> {
    let identity = restored.branch_identity(branch_id).map_err(|denial| {
        DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!("recovery branch identity admission denied: {denial:?}"),
        )
    })?;
    restored
        .admitted_branch_basis_for_identity(&identity)
        .map_err(|denial| {
            DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                format!("recovery branch binding admission denied: {denial:?}"),
            )
        })
}
