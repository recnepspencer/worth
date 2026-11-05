//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCurrentBodyRequirementAssessment {
    #[doc(hidden)]
    pub fn new_for_test(
        findings: Vec<WorthQueryOperationBodyRequirementFinding>,
        memory: crate::domain_computation::request_execution::QueryMemoryReservation,
    ) -> Self {
        Self::new(findings, memory)
    }
}
