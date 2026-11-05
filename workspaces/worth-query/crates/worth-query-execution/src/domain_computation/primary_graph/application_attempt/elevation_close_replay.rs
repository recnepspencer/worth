use crate::domain_computation::primary_graph::application_attempt::IdempotencyResolutionOps;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    closed_outcome, WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
    WorthQueryApplicationIdempotencyResolutionDenial, WorthQueryApprovedElevation,
    WorthQueryElevationCloseOutcome,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryPrimaryGraphApplicationRuntime,
};

pub trait ElevationCloseReplayOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    fn resolve_admitted_elevation_close_replay<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryElevationCloseOutcome>,
        (
            WorthQueryApplicationIdempotencyResolutionDenial,
            WorthQueryApprovedElevation,
        ),
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        if admission.elevation_close_binding().is_none() {
            return Ok(None);
        }
        let resolution = self
            .resolve_admitted_application_idempotency(phase, admission, idempotency)
            .map_err(|denial| {
                let approved = admission
                    .take_elevation_close_binding()
                    .expect("the elevation close binding was checked above")
                    .into_approved();
                (denial, approved)
            })?
            .into_resolution();
        let outcome = match resolution {
            WorthQueryApplicationIdempotencyResolution::Unseen => return Ok(None),
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
                WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt)
            }
            WorthQueryApplicationIdempotencyResolution::IntentDrift => {
                WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
                )
            }
        };
        let binding = admission
            .take_elevation_close_binding()
            .expect("the elevation close binding was checked above");
        Ok(Some(closed_outcome(outcome, binding)))
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ElevationCloseReplayOps<Schema> for HostRuntime {}
