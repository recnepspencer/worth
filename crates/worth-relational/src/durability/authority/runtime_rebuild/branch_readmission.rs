use crate::branch::basis_observation_inputs::BranchObservationInputs;
use crate::durability::data::{DurabilityError, RecoveryFailureClass};

pub(super) fn recovered_branch_basis(
    observations: BranchObservationInputs<'_>,
    branch_id: &crate::history::data::BranchId,
) -> Result<crate::branch::basis::AdmittedRelationalBranchBasis, DurabilityError> {
    let identity = observations.branch_identity(branch_id).map_err(|denial| {
        DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!("recovery branch identity admission denied: {denial:?}"),
        )
    })?;
    observations
        .observe(&identity, || None)
        .map(|(_, basis)| basis)
        .map_err(|denial| {
            DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                format!("recovery branch binding admission denied: {denial:?}"),
            )
        })
}

pub(super) fn owner_merge_parent_bases(
    observations: BranchObservationInputs<'_>,
    branches: &[crate::history::data::BranchId],
) -> Result<Vec<crate::branch::basis::AdmittedRelationalBranchBasis>, DurabilityError> {
    branches
        .iter()
        .map(|branch| {
            let identity = observations.branch_identity(branch).map_err(|denial| {
                DurabilityError::new(
                    RecoveryFailureClass::ReplayFailure,
                    format!("recovered merge parent identity was denied: {denial:?}"),
                )
            })?;
            observations
                .observe(&identity, || None)
                .map(|(_, basis)| basis)
                .map_err(|denial| {
                    DurabilityError::new(
                        RecoveryFailureClass::ReplayFailure,
                        format!("recovered merge parent binding was denied: {denial:?}"),
                    )
                })
        })
        .collect()
}
