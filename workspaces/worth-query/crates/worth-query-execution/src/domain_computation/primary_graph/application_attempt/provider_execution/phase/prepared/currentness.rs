use super::super::super::outcome::commit_outcome_from_authorization_denial;
use super::{
    denied, DenialStage, WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitOutcome, WorthQueryElevationCommitCurrentness,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) fn validate_operation_currentness<Schema, Operation, Input, Scope>(
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    admission.validate_current_authority().map_err(|denial| {
        commit_outcome_from_authorization_denial(denial, DenialStage::DecisionReadSet)
    })
}

pub(super) fn validate_elevation_currentness<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    if elevation_currentness
        .as_ref()
        .is_some_and(|currentness| !currentness.remains_current(&application.authorization_clock))
    {
        Err(denied(DenialStage::DecisionReadSet))
    } else {
        Ok(())
    }
}

/// A workflow step prepared before its instance's deadline still commits only
/// while the deadline lies ahead on the installed clock.
pub(super) fn validate_workflow_deadline<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    deadline: Option<u64>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    deadline.map_or(Ok(()), |deadline| {
        crate::domain_computation::primary_graph::application_attempt::workflow_deadline::ensure_before(
            &application.authorization_clock,
            deadline,
        )
        .map_err(|denial| {
            WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::workflow_settlement_denied(&denial),
            )
        })
    })
}
