//! Moved whole from `domain_computation::primary_graph::operation_body::assessment`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::operation_body::requirement::WorthQueryOperationBodyRequirementFinding;

/// Descriptive findings retain their native memory claim for their whole lifetime.
/// This result carries no projection snapshot or mutation authority.
pub struct WorthQueryCurrentBodyRequirementAssessment {
    findings: Vec<WorthQueryOperationBodyRequirementFinding>,
    _memory: Option<crate::domain_computation::request_execution::QueryMemoryReservation>,
}

impl WorthQueryCurrentBodyRequirementAssessment {
    pub fn findings(&self) -> &[WorthQueryOperationBodyRequirementFinding] {
        &self.findings
    }
    pub(in crate::domain_computation::primary_graph) fn new(
        findings: Vec<WorthQueryOperationBodyRequirementFinding>,
        memory: crate::domain_computation::request_execution::QueryMemoryReservation,
    ) -> Self {
        Self {
            findings,
            _memory: Some(memory),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn empty() -> Self {
        Self {
            findings: Vec::new(),
            _memory: None,
        }
    }
}

impl WorthQueryCurrentBodyRequirementAssessment {
    #[doc(hidden)]
    pub fn new_for_test(
        findings: Vec<WorthQueryOperationBodyRequirementFinding>,
        memory: crate::domain_computation::request_execution::QueryMemoryReservation,
    ) -> Self {
        Self::new(findings, memory)
    }
}
