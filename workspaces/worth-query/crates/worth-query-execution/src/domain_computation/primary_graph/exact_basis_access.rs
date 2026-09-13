use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    runtime::RelationalRuntime,
    snapshots::{RelationalSnapshotAdmissionDenial, SnapshotHandle},
};
#[cfg(test)]
use worth_relational::facade::{branch::RelationalBranchBasisDenial, history::BranchId};

pub(crate) fn open_exact_basis_snapshot(
    runtime: &mut RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<SnapshotHandle, RelationalSnapshotAdmissionDenial> {
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryExactBasisSnapshotDenial {
    #[cfg(test)]
    BranchIdentityUnavailable,
    BranchObservationUnavailable,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    SnapshotIdentityExhausted,
}

#[cfg(test)]
fn observation_denial(denial: RelationalBranchBasisDenial) -> WorthQueryExactBasisSnapshotDenial {
    match denial {
        RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted
        }
        RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted
        }
        _ => WorthQueryExactBasisSnapshotDenial::BranchObservationUnavailable,
    }
}

impl From<RelationalSnapshotAdmissionDenial> for WorthQueryExactBasisSnapshotDenial {
    fn from(denial: RelationalSnapshotAdmissionDenial) -> Self {
        match denial {
            RelationalSnapshotAdmissionDenial::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            } => Self::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            },
            RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted => {
                Self::SnapshotIdentityExhausted
            }
        }
    }
}

/// Opens an ephemeral snapshot from the Relational owner's exact current
/// branch observation. The descriptive branch name never selects storage on
/// its own.
#[cfg(test)]
pub(crate) fn open_current_branch_snapshot(
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
