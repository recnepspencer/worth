//! Typed decision-read refusals and their native physical cause.
use std::sync::Arc;
use worth_execution::ExecutionAllocationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDecisionReadSetDenialKind {
    InvalidRequest,
    UndeclaredFamily,
    FamilyKindMismatch,
    DecisionFactsUnsupported,
    InvalidProviderEvidence,
    EvidenceSubstitution,
    IncompleteRequiredFamilies,
    IncompleteRequiredFacts,
    DecisionFactBudgetExceeded,
    FactCountOverflow,
    RequestInterrupted(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    ),
    AllocationDenied,
    ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind),
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    ProviderRejected,
    ProviderPanicked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDecisionReadSetFailure {
    kind: WorthQueryDecisionReadSetDenialKind,
    detail: Arc<str>,
    allocation: Option<ExecutionAllocationDenial>,
    native_query_read_stop: Option<Arc<worth_relational::facade::runtime::QueryReadExecutionStop>>,
}

impl WorthQueryDecisionReadSetFailure {
    pub fn new(kind: WorthQueryDecisionReadSetDenialKind, detail: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            detail: detail.into(),
            allocation: None,
            native_query_read_stop: None,
        }
    }

    pub fn kind(&self) -> WorthQueryDecisionReadSetDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Preserve the physical owner's original kind and checked byte quote.
    pub fn allocation_denied(denial: ExecutionAllocationDenial) -> Self {
        Self {
            kind: WorthQueryDecisionReadSetDenialKind::AllocationDenied,
            detail: denial.to_string().into(),
            allocation: Some(denial),
            native_query_read_stop: None,
        }
    }

    pub(in crate::domain_computation) fn with_native_query_read_stop(
        mut self,
        stop: worth_relational::facade::runtime::QueryReadExecutionStop,
    ) -> Self {
        self.native_query_read_stop = Some(Arc::new(stop));
        self
    }

    /// The original native refusal, shared when the read-set failure is cloned.
    pub fn native_query_read_stop(
        &self,
    ) -> Option<&worth_relational::facade::runtime::QueryReadExecutionStop> {
        self.native_query_read_stop.as_deref()
    }

    pub fn allocation_denial(&self) -> Option<&ExecutionAllocationDenial> {
        self.allocation.as_ref()
    }
}
