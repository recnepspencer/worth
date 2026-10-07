/// Why an invariant projection could not take its snapshot or finish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantProjectionDenialKind {
    /// The main branch basis could not be observed.
    BasisUnavailable,
    /// The basis or snapshot belongs to a different runtime.
    ForeignBasis,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// No capacity remains to retain a basis, or the evidence a read of a
    /// current output retains.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The projection exceeded its work budget.
    WorkBudgetExceeded,
}

/// Refusal to run an invariant projection. No output or snapshot is returned;
/// [`Self::kind`] says why.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantProjectionDenial {
    kind: WorthQueryInvariantProjectionDenialKind,
}

impl WorthQueryInvariantProjectionDenial {
    pub const fn kind(&self) -> WorthQueryInvariantProjectionDenialKind {
        self.kind
    }

    pub(super) const fn basis_unavailable() -> Self {
        Self {
            kind: WorthQueryInvariantProjectionDenialKind::BasisUnavailable,
        }
    }

    pub(super) const fn from_kind(kind: WorthQueryInvariantProjectionDenialKind) -> Self {
        Self { kind }
    }

    pub(super) const fn work_budget_exceeded() -> Self {
        Self {
            kind: WorthQueryInvariantProjectionDenialKind::WorkBudgetExceeded,
        }
    }
}

impl std::fmt::Display for WorthQueryInvariantProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invariant projection denied: {:?}", self.kind)
    }
}

impl std::error::Error for WorthQueryInvariantProjectionDenial {}

pub(super) fn from_branch_basis_denial(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryInvariantProjectionDenial {
    match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::ForeignRuntime { .. } => {
            WorthQueryInvariantProjectionDenial::from_kind(
                WorthQueryInvariantProjectionDenialKind::ForeignBasis,
            )
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryInvariantProjectionDenial::from_kind(
                WorthQueryInvariantProjectionDenialKind::RetentionCapacityExhausted,
            )
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryInvariantProjectionDenial::from_kind(
                WorthQueryInvariantProjectionDenialKind::RetentionIdentityExhausted,
            )
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryInvariantProjectionDenial::from_kind(
                WorthQueryInvariantProjectionDenialKind::SnapshotIdentityExhausted,
            )
        }
        _ => WorthQueryInvariantProjectionDenial::basis_unavailable(),
    }
}

pub(super) fn from_snapshot_admission_denial(
    denial: worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial,
) -> WorthQueryInvariantProjectionDenial {
    let kind = match denial {
        worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryInvariantProjectionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ForeignRuntime { .. } => {
            WorthQueryInvariantProjectionDenialKind::ForeignBasis
        }
        worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted => {
            WorthQueryInvariantProjectionDenialKind::SnapshotIdentityExhausted
        }
    };
    WorthQueryInvariantProjectionDenial::from_kind(kind)
}
