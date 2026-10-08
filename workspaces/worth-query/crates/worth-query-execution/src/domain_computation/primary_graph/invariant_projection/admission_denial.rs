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
    /// No capacity remains to retain a basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The projection exceeded its work budget.
    WorkBudgetExceeded,
    /// Retaining full source predicates failed; its exact owner cause is kept.
    SourceRetentionDenied,
}

/// Refusal to run an invariant projection. No output or snapshot is returned;
/// [`Self::kind`] says why.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantProjectionDenial {
    kind: WorthQueryInvariantProjectionDenialKind,
    projection_work: Option<super::WorthQueryInvariantProjectionWork>,
    retention_denial: Option<crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial>,
}

impl WorthQueryInvariantProjectionDenial {
    pub const fn kind(&self) -> WorthQueryInvariantProjectionDenialKind {
        self.kind
    }

    /// Actual reader work before refusal; absent when no reader executed.
    pub const fn projection_work(&self) -> Option<super::WorthQueryInvariantProjectionWork> {
        self.projection_work
    }

    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        match &self.retention_denial {
            Some(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Allocation(denial)) => Some(denial),
            _ => None,
        }
    }
    pub fn source_retention_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match &self.retention_denial { Some(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::RequestInterruption(stop)) => Some(*stop), _ => None }
    }
    pub(super) fn source_retention_denied(
        denial: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
        work: super::WorthQueryInvariantProjectionWork,
    ) -> Self {
        Self {
            kind: WorthQueryInvariantProjectionDenialKind::SourceRetentionDenied,
            projection_work: Some(work),
            retention_denial: Some(denial),
        }
    }
    pub(super) fn with_retention_denial(
        mut self,
        denial: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
    ) -> Self {
        self.retention_denial = Some(denial);
        self
    }
    pub(super) const fn basis_unavailable() -> Self {
        Self {
            kind: WorthQueryInvariantProjectionDenialKind::BasisUnavailable,
            projection_work: None,
            retention_denial: None,
        }
    }

    pub(super) const fn from_kind(kind: WorthQueryInvariantProjectionDenialKind) -> Self {
        Self {
            kind,
            projection_work: None,
            retention_denial: None,
        }
    }

    pub(super) const fn work_budget_exceeded(
        work: super::WorthQueryInvariantProjectionWork,
    ) -> Self {
        Self {
            kind: WorthQueryInvariantProjectionDenialKind::WorkBudgetExceeded,
            projection_work: Some(work),
            retention_denial: None,
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
