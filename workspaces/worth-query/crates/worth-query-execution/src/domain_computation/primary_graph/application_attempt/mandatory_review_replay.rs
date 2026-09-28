use worth_query_installation::facade::ApplicationSchema;

use super::{
    reviewed_outcome, WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
    WorthQueryApplicationIdempotencyResolutionDenial, WorthQueryMandatoryReview,
    WorthQueryMandatoryReviewOutcome,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Resolve an exact mandatory-review replay before current-state program materialization.
    ///
    /// An admission that holds no mandatory review binding has no review replay: this
    /// returns `None` and the lane refuses it, typed, when it materializes.
    pub fn resolve_admitted_mandatory_review_replay<Operation, Input, Scope>(
        &self,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<WorthQueryMandatoryReviewOutcome>,
        (
            WorthQueryApplicationIdempotencyResolutionDenial,
            WorthQueryMandatoryReview,
        ),
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        if admission.mandatory_review_binding().is_none() {
            return Ok(None);
        }
        let resolution = self
            .resolve_admitted_application_idempotency(admission, idempotency)
            .map_err(|denial| {
                let mandatory = admission
                    .take_mandatory_review_binding()
                    .expect("the mandatory review binding was checked above")
                    .into_mandatory();
                (denial, mandatory)
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
            .take_mandatory_review_binding()
            .expect("the mandatory review binding was checked above");
        Ok(Some(reviewed_outcome(outcome, binding)))
    }
}
