use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationRequestExt, WorthQueryApplicationRetainedMutationOutcome,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveOutcome,
};
use worth_query_host::facade::publication::domain_computation::publish_application_result;

use crate::declaration::WorthUiStatusUpdateDenial;
use crate::{
    WorthUiScalarProjectionActionEvidence, WorthUiStatusActionRequest, WorthUiStatusPublication,
    WorthUiStatusQueryRequest,
};

use super::application_authentication::{authenticate, request_scope};
use super::WorthUiStatusSourceOwner;

pub struct WorthUiStatusActionExecution {
    publication: WorthUiStatusPublication,
    evidence: WorthUiScalarProjectionActionEvidence,
}

pub enum WorthUiStatusActionOutcome {
    Committed(WorthUiStatusActionExecution),
    DeniedStaleRevision {
        active_revision: u64,
        submitted_revision: u64,
    },
}

enum ActionSubmission {
    Provided(WorthUiStatusActionRequest),
    DeliberatelyStale {
        status: String,
        session: u64,
        lineage: u64,
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
    ) -> Result<WorthUiStatusActionOutcome, String> {
        self.execute_action_submission(ActionSubmission::Provided(action))
    }

    pub fn execute_stale_action(
        &self,
        status: impl Into<String>,
        session: u64,
        lineage: u64,
    ) -> Result<WorthUiStatusActionOutcome, String> {
        self.execute_action_submission(ActionSubmission::DeliberatelyStale {
            status: status.into(),
            session,
            lineage,
        })
    }

    fn execute_action_submission(
        &self,
        submission: ActionSubmission,
    ) -> Result<WorthUiStatusActionOutcome, String> {
        let scope = request_scope();
        let principal = authenticate(self.application.installed_schema(), &scope)?;
        let request = self.application.request(&principal, &scope);
        let before = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .execute()
            .map_err(|error| format!("{error:?}"))?;
        let [source] = before.observed_sources() else {
            return Err("the status action did not observe its unique source".to_owned());
        };
        let [row] = before.rows() else {
            return Err("the status action did not read its unique record".to_owned());
        };
        let active_revision = row.revision;
        let action = match submission {
            ActionSubmission::Provided(action) => action,
            ActionSubmission::DeliberatelyStale {
                status,
                session,
                lineage,
            } => WorthUiStatusActionRequest::new(
                active_revision.checked_add(1).unwrap_or(0),
                status,
                session,
                lineage,
            )
            .map_err(str::to_owned)?,
        };
        let source_revision = action.source_revision();
        let status = action.status().to_owned();
        let mut live = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1, 64))
            .map_err(|error| format!("{error:?}"))?;
        let result = (|| {
            let identity = action.identity();
            let outcome = request
                .mutate(action)
                .expect_source(source.clone())
                .idempotency(&identity)
                .execute_retained_in_program(&self.application)
                .map_err(|error| format!("{error:?}"))?;
            let retained = match outcome {
                WorthQueryApplicationRetainedMutationOutcome::Committed { retained, .. } => {
                    retained
                }
                WorthQueryApplicationRetainedMutationOutcome::Other(
                    WorthQueryApplicationMutationOutcome::DomainDenied(
                        WorthUiStatusUpdateDenial::StaleRevision,
                    ),
                ) => {
                    return Ok(WorthUiStatusActionOutcome::DeniedStaleRevision {
                        active_revision,
                        submitted_revision: source_revision,
                    });
                }
                WorthQueryApplicationRetainedMutationOutcome::Other(outcome) => {
                    return Err(format!("UI status action was not committed: {outcome:?}"));
                }
            };
            let outcome = live.next(&request).map_err(|error| format!("{error:?}"))?;
            let WorthQueryApplicationLiveOutcome::Delivered(update) = outcome else {
                return Err(
                    "the accepted status action did not deliver its live projection".to_owned(),
                );
            };
            if update.product_publication().composite_commit() != retained.selected_commit() {
                return Err("the action projection used a different committed product".to_owned());
            }
            let (_, admitted) = update.into_admitted_disclosed();
            let published = publish_application_result(admitted);
            let [row] = published.rows() else {
                return Err("the status action query did not return one record".to_owned());
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
        if !matches!(
            live.close(),
            WorthQueryApplicationLiveCloseOutcome::Completed(_)
        ) {
            return Err("the status action live subscription did not close".to_owned());
        };
        result
    }
}
