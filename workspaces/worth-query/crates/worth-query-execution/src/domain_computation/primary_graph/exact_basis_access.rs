use worth_relational::facade::{
    branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDenial},
    history::{BranchId, RelationalCommitReceipt},
    runtime::RelationalRuntime,
    snapshots::{RelationalSnapshotAdmissionDenial, SnapshotHandle},
};

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

pub(crate) fn open_current_main_snapshot(
    runtime: &mut RelationalRuntime,
) -> Result<SnapshotHandle, WorthQueryExactBasisSnapshotDenial> {
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .map_err(observation_denial)?;
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .map_err(Into::into)
}

/// Reads the current canonical head only through an owner-admitted repeatable
/// observation.
pub(crate) fn current_branch_head(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> Result<Option<RelationalCommitReceipt>, WorthQueryExactBasisSnapshotDenial> {
    let basis = current_branch_basis(runtime, branch)?;
    runtime
        .history()
        .branch_head_for_observation(&basis.observation())
        .map_err(|_| WorthQueryExactBasisSnapshotDenial::BranchObservationUnavailable)
}

/// Carries the owner-admitted immutable root when downstream work must use
/// more than the descriptive head receipt.
pub(crate) fn current_branch_basis(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> Result<
    worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    WorthQueryExactBasisSnapshotDenial,
> {
    let identity = runtime
        .branch_identity(branch)
        .map_err(|_| WorthQueryExactBasisSnapshotDenial::BranchIdentityUnavailable)?;
    let (_, basis) = runtime
        .observe_branch(&identity)
        .map_err(observation_denial)?;
    Ok(basis)
}
