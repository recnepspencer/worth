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

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Resolve an exact close replay before current-state program materialization.
    ///
    /// An admission that holds no elevation close binding has no close replay: this
    /// returns `None` and the lane refuses it, typed, when it materializes.
    pub fn resolve_admitted_elevation_close_replay<Operation, Input, Scope>(
        &self,
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
            .resolve_admitted_application_idempotency(admission, idempotency)
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
