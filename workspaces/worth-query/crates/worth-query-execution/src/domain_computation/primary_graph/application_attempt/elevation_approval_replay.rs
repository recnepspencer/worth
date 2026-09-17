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

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Resolve an exact approval replay before current-state program materialization.
    pub fn resolve_admitted_elevation_approval_replay<Operation, Input, Scope>(
        &self,
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
        let resolution = self
            .resolve_admitted_application_idempotency(admission, idempotency)
            .map_err(|denial| {
                let requested = admission
                    .take_elevation_approval_binding()
                    .expect("an admitted elevation approval retains its lifecycle binding")
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
            .expect("an admitted elevation approval retains its lifecycle binding");
        Ok(Some(approved_outcome(outcome, binding)))
    }
}
