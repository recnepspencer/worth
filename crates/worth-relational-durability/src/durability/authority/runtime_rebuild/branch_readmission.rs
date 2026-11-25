use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use worth_relational_branch::internal_port::branch::basis_observation_inputs::BranchObservationInputs;
use worth_relational_contracts::internal_port as contracts;

pub fn recovered_branch_basis(
    observations: BranchObservationInputs<'_>,
    branch_id: &contracts::history::data::coordinates::BranchId,
) -> Result<
    worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    DurabilityError,
> {
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

pub fn owner_merge_parent_bases(
    observations: BranchObservationInputs<'_>,
    branches: &[contracts::history::data::coordinates::BranchId],
) -> Result<
    Vec<worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis>,
    DurabilityError,
> {
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
