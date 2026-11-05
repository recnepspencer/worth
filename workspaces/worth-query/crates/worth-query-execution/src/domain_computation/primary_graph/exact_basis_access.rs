use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    runtime::RelationalRuntime,
    snapshots::{RelationalSnapshotAdmissionDenial, SnapshotHandle},
};
use worth_relational::facade::{branch::RelationalBranchBasisDenial, history::BranchId};
use crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial;

pub fn open_exact_basis_snapshot(
    runtime: &mut RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<SnapshotHandle, RelationalSnapshotAdmissionDenial> {
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
}


fn observation_denial(denial: RelationalBranchBasisDenial) -> WorthQueryExactBasisSnapshotDenial {
    match denial {
        RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted
        }
        RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted
        }
        RelationalBranchBasisDenial::MaterializationUnavailable => {
            WorthQueryExactBasisSnapshotDenial::BranchMaterializationSuspended
        }
        _ => WorthQueryExactBasisSnapshotDenial::BranchObservationUnavailable,
    }
}


/// Opens an ephemeral snapshot from the Relational owner's exact current
/// branch observation. The descriptive branch name never selects storage on
/// its own.
pub fn open_current_branch_snapshot(
    runtime: &mut RelationalRuntime,
    branch: &BranchId,
) -> Result<SnapshotHandle, WorthQueryExactBasisSnapshotDenial> {
    let identity = runtime
        .branch_identity(branch)
        .map_err(|_| WorthQueryExactBasisSnapshotDenial::BranchIdentityUnavailable)?;
    let (_, basis) = runtime
        .observe_branch(&identity)
        .map_err(observation_denial)?;
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .map_err(Into::into)
}
