//! Typed decision-read refusals and their native physical cause.
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionAllocationDenial};

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

impl ChargedBytes for WorthQueryDecisionReadSetFailure {
    fn additional_charged_bytes(&self) -> u64 {
        // Each retained Arc keeps its allocation alive, including both counts.
        // The error's inline fields (including the allocation denial) are
        // already part of the result slot; neither may be charged twice.
        let arc_header = std::mem::size_of::<[usize; 2]>() as u64;
        let detail = arc_header + self.detail.len() as u64;
        let native = self.native_query_read_stop.as_ref().map_or(0, |stop| {
            arc_header
                + std::mem::size_of_val(stop.as_ref()) as u64
                + stop.additional_charged_bytes()
        });
        detail + native + self.allocation.additional_charged_bytes()
    }
}

#[cfg(test)]
mod footprint_tests {
    use super::*;

    #[test]
    fn decision_failure_retains_its_original_detail_and_native_cause_allocations() {
        let detail: Arc<str> = Arc::from("original owned admission detail");
        let failure = WorthQueryDecisionReadSetFailure::new(
            WorthQueryDecisionReadSetDenialKind::ProviderRejected,
            Arc::clone(&detail),
        );
        let header = std::mem::size_of::<[usize; 2]>() as u64;
        assert!(Arc::ptr_eq(&failure.detail, &detail));
        assert_eq!(
            failure.additional_charged_bytes(),
            header + detail.len() as u64
        );
        let failure = failure.with_native_query_read_stop(
            worth_relational::facade::runtime::QueryReadExecutionStop::ReadRequest(
                worth_relational::facade::indexes::RelationalExecutionDenialCause::Cancelled,
            ),
        );
        let native = failure.native_query_read_stop.as_ref().unwrap();
        assert_eq!(
            failure.additional_charged_bytes(),
            header
                + detail.len() as u64
                + header
                + std::mem::size_of_val(native.as_ref()) as u64
                + native.additional_charged_bytes(),
        );
        assert!(Arc::ptr_eq(&failure.detail, &detail));
    }
}
