use crate::domain_computation::primary_graph::application_attempt::IdempotencyResolutionOps;
use worth_query_installation::facade::ApplicationSchema;

use {super::reviewed_outcome, super::WorthQueryApplicationCommitDenial, super::WorthQueryApplicationCommitOutcome, crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding, super::WorthQueryApplicationIdempotencyResolution, super::WorthQueryApplicationIdempotencyResolutionDenial, super::WorthQueryMandatoryReview, super::WorthQueryMandatoryReviewOutcome};
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;

pub trait MandatoryReviewReplayOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    fn resolve_admitted_mandatory_review_replay<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
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
            .resolve_admitted_application_idempotency(phase, admission, idempotency)
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

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> MandatoryReviewReplayOps<Schema> for HostRuntime {}
