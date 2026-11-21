//! Read-only assessment borrows resources from the exact native runtime phase.
use super::WorthQueryOperationBodyRequirementFinding;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryForeignAdvancementPhase,
    WorthQueryOperationProjectionDenial,
};

/// Descriptive findings retain their native memory claim for their whole lifetime.
/// This result carries no projection snapshot or mutation authority.
pub struct WorthQueryCurrentBodyRequirementAssessment {
    findings: Vec<WorthQueryOperationBodyRequirementFinding>,
    _memory: Option<
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    >,
}

impl WorthQueryCurrentBodyRequirementAssessment {
    pub fn findings(&self) -> &[WorthQueryOperationBodyRequirementFinding] {
        &self.findings
    }
    pub fn new(
        findings: Vec<WorthQueryOperationBodyRequirementFinding>,
        memory: crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    ) -> Self {
        Self {
            findings,
            _memory: Some(memory),
        }
    }
    pub fn empty() -> Self {
        Self {
            findings: Vec::new(),
            _memory: None,
        }
    }
}

/// A native body assessment preserves phase, projection and observation refusal.
#[derive(Debug)]
pub enum WorthQueryOperationBodyAssessmentDenial {
    Source(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
    Phase(WorthQueryForeignAdvancementPhase),
    Projection(WorthQueryOperationProjectionDenial),
    Observation(HandlerExecutionDenial),
}

mod court_doors_for_test;
