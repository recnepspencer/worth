use crate::domain_computation::primary_graph::application_attempt::IdempotencyResolutionOps;
use crate::domain_computation::primary_graph::application_attempt::IdempotencyResolutionScopedOps;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    approved_outcome, WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
    WorthQueryApplicationIdempotencyResolutionDenial, WorthQueryElevationApprovalOutcome,
    WorthQueryRequestedElevation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryPrimaryGraphApplicationRuntime,
};

pub trait ElevationApprovalReplayOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    fn resolve_admitted_elevation_approval_replay<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryElevationApprovalOutcome>,
        (
            WorthQueryApplicationIdempotencyResolutionDenial,
            WorthQueryRequestedElevation,
        ),
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        if admission.elevation_approval_binding().is_none() {
            return Ok(None);
        }
        let resolution = self
            .resolve_admitted_application_idempotency(phase, admission, idempotency)
            .map_err(|denial| {
                let requested = admission
                    .take_elevation_approval_binding()
                    .expect("the elevation approval binding was checked above")
                    .into_requested();
                (denial, requested)
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
            .take_elevation_approval_binding()
            .expect("the elevation approval binding was checked above");
        Ok(Some(approved_outcome(outcome, binding)))
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ElevationApprovalReplayOps<Schema> for HostRuntime {}

pub(in crate::domain_computation) trait ElevationApprovalReplayScopedOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    fn resolve_admitted_elevation_approval_replay_with_request<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryElevationApprovalOutcome>,
        (
            WorthQueryApplicationIdempotencyResolutionDenial,
            WorthQueryRequestedElevation,
        ),
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        if admission.elevation_approval_binding().is_none() {
            return Ok(None);
        }
        let resolution = self
            .resolve_admitted_application_idempotency_with_request(
                execution,
                admission,
                idempotency,
            )
            .map_err(|denial| {
                let requested = admission
                    .take_elevation_approval_binding()
                    .expect("the elevation approval binding was checked above")
                    .into_requested();
                (denial, requested)
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
            .take_elevation_approval_binding()
            .expect("the elevation approval binding was checked above");
        Ok(Some(approved_outcome(outcome, binding)))
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ElevationApprovalReplayScopedOps<Schema> for HostRuntime {}
