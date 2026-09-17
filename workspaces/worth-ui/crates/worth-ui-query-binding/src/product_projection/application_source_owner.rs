use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRetainedMutationOutcome,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveOutcome,
};
use worth_query_host::facade::publication::domain_computation::{
    publish_application_result, WorthQueryApplicationQueryPublicationReceipt,
};

use crate::{
    UiApplicationScalarProjectionRegistration, UiProjectionObservation,
    WorthUiScalarProjectionSourceRecord, WorthUiStatusQueryRequest, WorthUiStatusQueryResult,
};

use super::application_authentication::{authenticate, request_scope};
use super::application_runtime::{install_status_application, WorthUiStatusApplication};

pub struct WorthUiStatusSourceOwner {
    pub(super) application: WorthUiStatusApplication,
}

pub struct WorthUiStatusOwnerCloseReceipt {
    owner_terminal: bool,
}

impl WorthUiStatusOwnerCloseReceipt {
    pub const fn owner_terminal(&self) -> bool {
        self.owner_terminal
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorthUiStatusPublication {
    value: WorthUiStatusQueryResult,
    query_receipt: WorthQueryApplicationQueryPublicationReceipt,
}

impl WorthUiStatusPublication {
    pub(super) fn from_parts(
        value: WorthUiStatusQueryResult,
        query_receipt: WorthQueryApplicationQueryPublicationReceipt,
    ) -> Self {
        Self {
            value,
            query_receipt,
        }
    }
    pub fn value(&self) -> &WorthUiStatusQueryResult {
        &self.value
    }

    pub fn query_receipt(&self) -> &WorthQueryApplicationQueryPublicationReceipt {
        &self.query_receipt
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthUiStatusQueryResult,
        WorthQueryApplicationQueryPublicationReceipt,
    ) {
        (self.value, self.query_receipt)
    }
}

impl WorthUiStatusSourceOwner {
    pub fn close(self) -> WorthUiStatusOwnerCloseReceipt {
        drop(self);
        WorthUiStatusOwnerCloseReceipt {
            owner_terminal: true,
        }
    }

    pub fn install() -> Result<Self, String> {
        Ok(Self {
            application: install_status_application()?,
        })
    }

    pub fn read_status(&self) -> Result<WorthUiStatusPublication, String> {
        let scope = request_scope();
        let principal = authenticate(self.application.installed_schema(), &scope)?;
        let request = self.application.request(&principal, &scope);
        let result = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .execute()
            .map_err(|error| format!("{error:?}"))?;
        let [row] = result.rows() else {
            return Err("the UI status query did not return its unique record".to_owned());
        };
        Ok(WorthUiStatusPublication {
            value: row.clone(),
            query_receipt: result.receipt().clone(),
        })
    }

    pub fn initial_projection(
        &self,
    ) -> Result<
        (
            UiApplicationScalarProjectionRegistration,
            UiProjectionObservation,
        ),
        String,
    > {
        let publication = self.read_status()?;
        let registration = UiApplicationScalarProjectionRegistration::query_issued(&publication)
            .map_err(|error| format!("{error:?}"))?;
        let observation = publication
            .into_projection_observation()
            .map_err(|error| format!("{error:?}"))?;
        Ok((registration, observation))
    }

    pub fn publish_source(
        &self,
        record: WorthUiScalarProjectionSourceRecord,
    ) -> Result<WorthUiStatusPublication, String> {
        let scope = request_scope();
        let principal = authenticate(self.application.installed_schema(), &scope)?;
        let request = self.application.request(&principal, &scope);
        let before = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .execute()
            .map_err(|error| format!("{error:?}"))?;
        let [source] = before.observed_sources() else {
            return Err("the UI status query did not observe its unique source".to_owned());
        };
        let mut live = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1, 64))
            .map_err(|error| format!("{error:?}"))?;
        let result = (|| {
            let outcome = request
                .mutate(record.clone())
                .expect_source(source.clone())
                .idempotency(&record.revision())
                .execute_retained_in_program(&self.application)
                .map_err(|error| format!("{error:?}"))?;
            let retained = match outcome {
                WorthQueryApplicationRetainedMutationOutcome::Committed { retained, .. } => {
                    retained
                }
                WorthQueryApplicationRetainedMutationOutcome::Other(outcome) => {
                    return Err(format!(
                        "UI status source update was not committed: {outcome:?}"
                    ));
                }
            };
            let outcome = live.next(&request).map_err(|error| format!("{error:?}"))?;
            let WorthQueryApplicationLiveOutcome::Delivered(update) = outcome else {
                return Err(
                    "the accepted UI status edit did not deliver its live projection".to_owned(),
                );
            };
            if update.product_publication().composite_commit() != retained.selected_commit() {
                return Err(
                    "the live UI status projection used a different committed product".to_owned(),
                );
            }
            let (_, admitted) = update.into_admitted_disclosed();
            let published = publish_application_result(admitted);
            let [row] = published.rows() else {
                return Err("the live UI status query did not return one record".to_owned());
            };
            Ok(WorthUiStatusPublication {
                value: row.clone(),
                query_receipt: published.receipt().clone(),
            })
        })();
        if !matches!(
            live.close(),
            WorthQueryApplicationLiveCloseOutcome::Completed(_)
        ) {
            return Err("the UI status live subscription did not close".to_owned());
        };
        result
    }
}
