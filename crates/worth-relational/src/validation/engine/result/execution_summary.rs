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
    pub(crate) fn from_results(results: &[InvariantCheckResult]) -> Self {
        let mut advisory_count = 0;
        let mut violation_count = 0;
        let mut custom_failure_count = 0;
        let mut custom_panic_count = 0;
        let mut blocking_failure = None;
        let mut publication_failure = None;

        for result in results {
            match &result.verdict {
                InvariantVerdict::Pass | InvariantVerdict::NotApplicable => {}
                InvariantVerdict::Advisory { .. } => {
                    advisory_count += 1;
                }
                InvariantVerdict::Violation(violation) => {
                    violation_count += 1;
                    if let InvariantViolationFields::CustomInvariantFailure { failure, .. } =
                        &violation.fields
                    {
                        custom_failure_count += 1;
                        if *failure
                            == crate::validation::data::ResultCustomInvariantFailureKind::Panic
                        {
                            custom_panic_count += 1;
                        }
                    }
                    let failure = InvariantFailure::new(
                        result.execution_point,
                        result.failure_effect,
                        violation.clone(),
                    );
                    match result.failure_effect {
                        InvariantFailureEffect::BlockCommit => {
                            if blocking_failure.is_none() {
                                blocking_failure = Some(failure);
                            }
                        }
                        InvariantFailureEffect::BlockPublication => {
                            if publication_failure.is_none() {
                                publication_failure = Some(failure);
                            }
                        }
                        InvariantFailureEffect::AuditOnly => {}
                    }
                }
            }
        }

        Self {
            result_count: results.len(),
            advisory_count,
            violation_count,
            custom_failure_count,
            custom_panic_count,
            blocking_failure,
            publication_failure,
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
