use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationRequestExt, WorthQueryApplicationRetainedMutationOutcome,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome;
use worth_query_host::facade::publication::domain_computation::publish_application_result;

use crate::declaration::WorthUiStatusUpdateDenial;
use crate::{
    WorthUiScalarProjectionActionEvidence, WorthUiStatusActionRequest, WorthUiStatusPublication,
    WorthUiStatusQueryRequest,
};

use super::application_authentication::{authenticate, request_scope};
use super::status_owner_error::classify_live_delivery;
use super::WorthUiStatusOwnerError;
use super::WorthUiStatusSourceOwner;

#[derive(Debug)]
pub struct WorthUiStatusActionExecution {
    publication: WorthUiStatusPublication,
    evidence: WorthUiScalarProjectionActionEvidence,
}

#[derive(Debug)]
pub enum WorthUiStatusActionOutcome {
    Committed(WorthUiStatusActionExecution),
    DeniedRevisionMismatch {
        active_revision: u64,
        submitted_revision: u64,
        live_close: Option<WorthQueryApplicationLiveCloseOutcome>,
    },
}

impl WorthUiStatusActionExecution {
    pub fn into_parts(
        self,
    ) -> (
        WorthUiStatusPublication,
        WorthUiScalarProjectionActionEvidence,
    ) {
        (self.publication, self.evidence)
    }
}

impl WorthUiStatusSourceOwner {
    pub fn execute_action(
        &self,
        action: WorthUiStatusActionRequest,
    ) -> Result<WorthUiStatusActionOutcome, WorthUiStatusOwnerError> {
        let scope = request_scope();
        let principal = authenticate(self.application.installed_schema(), &scope)?;
        let request = self.application.request(&principal, &scope);
        let before = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .execute()
            .map_err(|error| WorthUiStatusOwnerError::Query(format!("{error:?}")))?;
        let [source] = before.observed_sources() else {
            return Err(WorthUiStatusOwnerError::MissingUniqueSource);
        };
        let [row] = before.rows() else {
            return Err(WorthUiStatusOwnerError::MissingUniqueRecord);
        };
        let active_revision = row.revision;
        let source_revision = action.source_revision();
        let status = action.status().to_owned();
        let mut live = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1, 64))
            .map_err(|error| WorthUiStatusOwnerError::LiveOpen(format!("{error:?}")))?;
        let result = (|| {
            let identity = action.identity();
            let outcome = request
                .mutate(action)
                .expect_source(source.clone())
                .idempotency(&identity)
                .execute_retained_in_program(&self.application)
                .map_err(|error| WorthUiStatusOwnerError::MutationRequest(format!("{error:?}")))?;
            let retained = match outcome {
                WorthQueryApplicationRetainedMutationOutcome::Committed { retained, .. } => {
                    retained
                }
                WorthQueryApplicationRetainedMutationOutcome::Other(
                    WorthQueryApplicationMutationOutcome::DomainDenied(
                        WorthUiStatusUpdateDenial::RevisionMismatch,
                    ),
                ) => {
                    return Ok(WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                        active_revision,
                        submitted_revision: source_revision,
                        live_close: None,
                    });
                }
                WorthQueryApplicationRetainedMutationOutcome::Other(outcome) => {
                    return Err(WorthUiStatusOwnerError::ActionMutationOutcome(Box::new(
                        outcome,
                    )))
                }
            };
            let outcome = live
                .next(&request)
                .map_err(|error| WorthUiStatusOwnerError::Query(format!("{error:?}")))?;
            let update = classify_live_delivery(outcome)?;
            if update.product_publication().composite_commit() != retained.selected_commit() {
                return Err(WorthUiStatusOwnerError::PublicationMismatch);
            }
            let (_, admitted) = update.into_admitted_disclosed();
            let published = publish_application_result(admitted);
            let [row] = published.rows() else {
                return Err(WorthUiStatusOwnerError::MissingUniqueRecord);
            };
            let publication =
                WorthUiStatusPublication::from_parts(row.clone(), published.receipt().clone());
            Ok(WorthUiStatusActionOutcome::Committed(
                WorthUiStatusActionExecution {
                    evidence: WorthUiScalarProjectionActionEvidence::from_application_publication(
                        source_revision,
                        status,
                        &publication,
                    ),
                    publication,
                },
            ))
        })();
        let close = live.close();
        match (result, close) {
            (Ok(WorthUiStatusActionOutcome::Committed(mut execution)), close) => {
                execution.publication = execution.publication.with_live_close(close);
                Ok(WorthUiStatusActionOutcome::Committed(execution))
            }
            (
                Ok(WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                    active_revision,
                    submitted_revision,
                    ..
                }),
                close,
            ) => Ok(WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                active_revision,
                submitted_revision,
                live_close: Some(close),
            }),
            (Err(error), WorthQueryApplicationLiveCloseOutcome::Completed(_)) => Err(error),
            (Err(error), close) => Err(error.with_live_close(close)),
        }
    }
}
