use crate::validation::data::{
    InvariantCheckResult, InvariantFailureEffect, InvariantVerdict, InvariantViolationFields,
};

use super::failures::InvariantFailure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantExecutionSummary {
    result_count: usize,
    advisory_count: usize,
    violation_count: usize,
    custom_failure_count: usize,
    custom_panic_count: usize,
    blocking_failure: Option<InvariantFailure>,
    publication_failure: Option<InvariantFailure>,
}

impl InvariantExecutionSummary {
    pub(crate) fn empty() -> Self {
        Self {
            result_count: 0,
            advisory_count: 0,
            violation_count: 0,
            custom_failure_count: 0,
            custom_panic_count: 0,
            blocking_failure: None,
            publication_failure: None,
        }
    }

    pub(crate) fn from_results(results: &[InvariantCheckResult]) -> Self {
        let mut summary = Self::empty();
        for result in results {
            summary.observe(result);
        }
        summary
    }

    /// The same classification for whole-result and incremental reduction.
    pub(crate) fn observe(&mut self, result: &InvariantCheckResult) {
        self.result_count += 1;
        match &result.verdict {
            InvariantVerdict::Pass | InvariantVerdict::NotApplicable => {}
            InvariantVerdict::Advisory { .. } => self.advisory_count += 1,
            InvariantVerdict::Violation(violation) => {
                self.violation_count += 1;
                if let InvariantViolationFields::CustomInvariantFailure { failure, .. } =
                    &violation.fields
                {
                    self.custom_failure_count += 1;
                    if *failure == crate::validation::data::ResultCustomInvariantFailureKind::Panic
                    {
                        self.custom_panic_count += 1;
                    }
                }
                let first = match result.failure_effect {
                    InvariantFailureEffect::BlockCommit => Some(&mut self.blocking_failure),
                    InvariantFailureEffect::BlockPublication => Some(&mut self.publication_failure),
                    InvariantFailureEffect::AuditOnly => None,
                };
                if let Some(first) = first
                    && first.is_none()
                {
                    *first = Some(InvariantFailure::new(
                        result.execution_point,
                        result.failure_effect,
                        violation.clone(),
                    ));
                }
            }
        }
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.blocking_failure
            .iter()
            .chain(&self.publication_failure)
            .fold(0, |total, failure| {
                total.saturating_add(failure.owned_allocation_capacity_bytes())
            })
    }

    /// Backing the next observation would retain; admission precedes its clone.
    pub(crate) fn pending_failure_allocation_bytes(&self, result: &InvariantCheckResult) -> u64 {
        let retains = match result.failure_effect {
            InvariantFailureEffect::BlockCommit => self.blocking_failure.is_none(),
            InvariantFailureEffect::BlockPublication => self.publication_failure.is_none(),
            InvariantFailureEffect::AuditOnly => false,
        };
        match &result.verdict {
            InvariantVerdict::Violation(violation) if retains => (violation.detail.capacity()
                as u64)
                .saturating_add(violation.fields.owned_allocation_capacity_bytes()),
            _ => 0,
        }
    }

    pub fn result_count(&self) -> usize {
        self.result_count
    }

    pub fn advisory_count(&self) -> usize {
        self.advisory_count
    }

    pub fn violation_count(&self) -> usize {
        self.violation_count
    }

    pub fn custom_failure_count(&self) -> usize {
        self.custom_failure_count
    }

    pub fn custom_panic_count(&self) -> usize {
        self.custom_panic_count
    }

    pub fn blocking_failure(&self) -> Option<&InvariantFailure> {
        self.blocking_failure.as_ref()
    }

    pub fn blocking_failures(&self, results: &[InvariantCheckResult]) -> Vec<InvariantFailure> {
        results
            .iter()
            .filter_map(|result| match &result.verdict {
                InvariantVerdict::Violation(violation)
                    if result.failure_effect == InvariantFailureEffect::BlockCommit =>
                {
                    Some(InvariantFailure::new(
                        result.execution_point,
                        result.failure_effect,
                        violation.clone(),
                    ))
                }
                _ => None,
            })
            .collect()
    }

    pub fn publication_failure(&self) -> Option<&InvariantFailure> {
        self.publication_failure.as_ref()
    }

    pub fn publication_failures(&self, results: &[InvariantCheckResult]) -> Vec<InvariantFailure> {
        results
            .iter()
            .filter_map(|result| match &result.verdict {
                InvariantVerdict::Violation(violation)
                    if result.failure_effect == InvariantFailureEffect::BlockPublication =>
                {
                    Some(InvariantFailure::new(
                        result.execution_point,
                        result.failure_effect,
                        violation.clone(),
                    ))
                }
                _ => None,
            })
            .collect()
    }

    pub fn has_blocking_violation(&self) -> bool {
        self.blocking_failure.is_some()
    }

    pub fn has_publication_violation(&self) -> bool {
        self.publication_failure.is_some()
    }
}
