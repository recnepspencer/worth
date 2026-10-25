/// Why an invariant projection could not take its snapshot or finish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantProjectionDenialKind {
    /// An admitted field decode refused; no callback output is accepted.
    ValueDecode(super::WorthQueryValueDecodeDenial),
    /// A required native read stopped; its exact request cause is retained.
    ExecutionRequest(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    /// The main branch basis could not be observed.
    BasisUnavailable,
    ForeignPhase,
    /// The basis or snapshot belongs to a different runtime.
    ForeignBasis,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// No capacity remains to retain a basis, or the evidence a read of a
    /// current output retains.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The projection exceeded its work budget.
    WorkBudgetExceeded,
    /// Retaining full source predicates failed; its exact owner cause is kept.
    SourceRetentionDenied,
}

/// Refusal to run an invariant projection. No output or snapshot is returned;
/// [`Self::kind`] says why. The boxed kind bounds the stack to work evidence
/// and two pointer words.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantProjectionDenial {
    kind: Box<WorthQueryInvariantProjectionDenialKind>,
    projection_work: Option<super::WorthQueryInvariantProjectionWork>,
    retention_denial: Option<Box<crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial>>,
}

const PROJECTION_DENIAL_STACK_BOUND: usize = std::mem::size_of::<
    Option<super::WorthQueryInvariantProjectionWork>,
>() + 2 * std::mem::size_of::<usize>();
const _: () = assert!(
    std::mem::size_of::<WorthQueryInvariantProjectionDenial>() <= PROJECTION_DENIAL_STACK_BOUND
);

impl WorthQueryInvariantProjectionDenial {
    pub(super) fn value_decode_stopped(
        cause: super::WorthQueryValueDecodeDenial,
        work: Option<super::WorthQueryInvariantProjectionWork>,
    ) -> Self {
        if let Some(worth_execution::WorkCeilingDenial::Stopped(stop)) = cause.request_cause() {
            return Self::execution_request_stopped(stop.into(), work);
        }
        Self {
            kind: Box::new(WorthQueryInvariantProjectionDenialKind::ValueDecode(cause)),
            projection_work: work,
            retention_denial: None,
        }
    }
    pub(super) fn execution_request_stopped(
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
        work: Option<super::WorthQueryInvariantProjectionWork>,
    ) -> Self {
        Self {
            kind: Box::new(WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                cause,
            )),
            projection_work: work,
            retention_denial: None,
        }
    }
    pub const fn kind(&self) -> WorthQueryInvariantProjectionDenialKind {
        *self.kind
    }

    /// Actual reader work before refusal; absent when no reader executed.
    pub const fn projection_work(&self) -> Option<super::WorthQueryInvariantProjectionWork> {
        self.projection_work
    }

    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        match self.retention_denial.as_deref() {
            Some(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Allocation(denial)) => Some(denial),
            _ => None,
        }
    }
    pub fn source_retention_interruption(
        &self,
    ) -> Option<worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption>
    {
        match self.retention_denial.as_deref() { Some(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::RequestInterruption(stop)) => Some(*stop), _ => None }
    }
    pub(super) fn source_retention_denied(
        denial: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
        work: super::WorthQueryInvariantProjectionWork,
    ) -> Self {
        Self {
            kind: Box::new(WorthQueryInvariantProjectionDenialKind::SourceRetentionDenied),
            projection_work: Some(work),
            retention_denial: Some(Box::new(denial)),
        }
    }
    pub(super) fn with_retention_denial(
        mut self,
        denial: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
    ) -> Self {
        self.retention_denial = Some(Box::new(denial));
        self
    }
    pub(super) fn basis_unavailable() -> Self {
        Self {
            kind: Box::new(WorthQueryInvariantProjectionDenialKind::BasisUnavailable),
            projection_work: None,
            retention_denial: None,
        }
    }

    pub(super) fn from_kind(kind: WorthQueryInvariantProjectionDenialKind) -> Self {
        Self {
            kind: Box::new(kind),
            projection_work: None,
            retention_denial: None,
        }
    }

    pub(super) fn work_budget_exceeded(work: super::WorthQueryInvariantProjectionWork) -> Self {
        Self {
            kind: Box::new(WorthQueryInvariantProjectionDenialKind::WorkBudgetExceeded),
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

#[cfg(test)]
mod stack_size_tests {
    use super::*;
    #[test]
    fn projection_request_stop_keeps_the_denial_stack_bound() {
        use std::mem::size_of;
        let bound = size_of::<Option<super::super::WorthQueryInvariantProjectionWork>>()
            + 2 * size_of::<usize>();
        assert!(size_of::<WorthQueryInvariantProjectionDenial>() <= bound);
        let cause =
            worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled;
        assert_eq!(
            WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None).kind(),
            WorthQueryInvariantProjectionDenialKind::ExecutionRequest(cause)
        );
    }
}
