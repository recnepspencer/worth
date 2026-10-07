use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure,
};

use super::material::ApplicationInvariantCandidateMaterial;

/// Reserves the complete declared custom execution set before entering Relational.
/// Actual consumed work is obtained separately from the validated candidate.
pub(super) fn admit_candidate_validator_work(
    material: &ApplicationInvariantCandidateMaterial,
    host_budget: Option<u64>,
) -> Result<u64, WorthQueryInvariantExecutionFailure> {
    let semantic_work =
        u64::try_from(material.semantic.expected.len()).map_err(|_| super::owner_failure())?;
    let required_work = material.requirements.iter().try_fold(
        semantic_work,
        |total, requirement| match requirement.application_invariant() {
            Some(invariant) => total
                .checked_add(invariant.maximum_work_units().get())
                .ok_or_else(super::owner_failure),
            None => Ok(total),
        },
    )?;
    let binding_budget = material
        .validator_work_admission
        .maximum_work()
        .map(|value| value as u64);
    let request_budget = material
        .request
        .candidate_validator_work_budget()
        .map(|value| value.get());
    let maximum = [binding_budget, request_budget, host_budget]
        .into_iter()
        .flatten()
        .min();
    if let Some(maximum) = maximum {
        let maximum_work = usize::try_from(maximum).map_err(|_| super::owner_failure())?;
        let required_work = usize::try_from(required_work).map_err(|_| super::owner_failure())?;
        if required_work > maximum_work {
            return Err(WorthQueryInvariantExecutionFailure::new(
                WorthQueryInvariantExecutionDenialKind::CandidateValidatorWorkExceeded {
                    maximum_work,
                    required_work,
                },
                "candidate invariant closure exceeds its requested work reservation",
            ));
        }
    }
    Ok(semantic_work)
}
