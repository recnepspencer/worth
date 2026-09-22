use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRetainedMutationOutcome,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome;
use worth_query_host::facade::publication::domain_computation::{
    publish_application_result, WorthQueryApplicationQueryPublicationReceipt,
};

use crate::{
    UiApplicationScalarProjectionRegistration, UiProjectionObservation,
    WorthUiScalarProjectionSourceRecord, WorthUiStatusQueryRequest, WorthUiStatusQueryResult,
};

use super::application_authentication::{authenticate, request_scope};
use super::application_runtime::{install_status_application, WorthUiStatusApplication};
use super::status_owner_error::classify_live_delivery;
use super::WorthUiStatusOwnerError;

pub struct WorthUiStatusSourceOwner {
    pub(super) application: WorthUiStatusApplication,
}

pub struct WorthUiStatusOwnerCloseReceipt {
    owner_terminal: bool,
    remaining_live_consumers: usize,
}

impl WorthUiStatusOwnerCloseReceipt {
    pub const fn owner_terminal(&self) -> bool {
        self.owner_terminal
    }

    pub const fn remaining_live_consumers(&self) -> usize {
        self.remaining_live_consumers
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorthUiStatusPublication {
    value: WorthUiStatusQueryResult,
    query_receipt: WorthQueryApplicationQueryPublicationReceipt,
    live_close: Option<WorthQueryApplicationLiveCloseOutcome>,
}

impl WorthUiStatusPublication {
    pub(super) fn from_parts(
        value: WorthUiStatusQueryResult,
        query_receipt: WorthQueryApplicationQueryPublicationReceipt,
    ) -> Self {
        Self {
            value,
            query_receipt,
            live_close: None,
        }
    }
    pub fn value(&self) -> &WorthUiStatusQueryResult {
        &self.value
    }

    pub fn query_receipt(&self) -> &WorthQueryApplicationQueryPublicationReceipt {
        &self.query_receipt
    }

    pub fn live_close(&self) -> Option<&WorthQueryApplicationLiveCloseOutcome> {
        self.live_close.as_ref()
    }

    pub(super) fn with_live_close(
        mut self,
        live_close: WorthQueryApplicationLiveCloseOutcome,
    ) -> Self {
        self.live_close = Some(live_close);
        self
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
        let close = self.application.close_live_delivery();
        let remaining_live_consumers = close.remaining_live_consumers();
        drop(self);
        WorthUiStatusOwnerCloseReceipt {
            owner_terminal: close.owner_terminal(),
            remaining_live_consumers,
        }
    }

    pub fn install() -> Result<Self, WorthUiStatusOwnerError> {
        Ok(Self {
            application: install_status_application()?,
        })
    }

    pub fn read_status(&self) -> Result<WorthUiStatusPublication, WorthUiStatusOwnerError> {
        let scope = request_scope();
        let principal = authenticate(self.application.installed_schema(), &scope)?;
        let request = self.application.request(&principal, &scope);
        let result = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .execute()
            .map_err(|error| WorthUiStatusOwnerError::Query(format!("{error:?}")))?;
        let [row] = result.rows() else {
            return Err(WorthUiStatusOwnerError::MissingUniqueRecord);
        };
        Ok(WorthUiStatusPublication {
            value: row.clone(),
            query_receipt: result.receipt().clone(),
            live_close: None,
        })
    }

    pub fn initial_projection(
        &self,
    ) -> Result<
        (
            UiApplicationScalarProjectionRegistration,
            UiProjectionObservation,
        ),
        WorthUiStatusOwnerError,
    > {
        let publication = self.read_status()?;
        let registration = UiApplicationScalarProjectionRegistration::query_issued(&publication)
            .map_err(|error| WorthUiStatusOwnerError::Query(format!("{error:?}")))?;
        let observation = publication
            .into_projection_observation()
            .map_err(|error| WorthUiStatusOwnerError::Query(format!("{error:?}")))?;
        Ok((registration, observation))
    }

    pub fn publish_source(
        &self,
        record: WorthUiScalarProjectionSourceRecord,
    ) -> Result<WorthUiStatusPublication, WorthUiStatusOwnerError> {
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
        let mut live = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1, 64))
            .map_err(|error| WorthUiStatusOwnerError::LiveOpen(format!("{error:?}")))?;
        let result = (|| {
            let outcome = request
                .mutate(record.clone())
                .expect_source(source.clone())
                .idempotency(&record.revision())
                .execute_retained_in_program(&self.application)
                .map_err(|error| WorthUiStatusOwnerError::MutationRequest(format!("{error:?}")))?;
            let retained = match outcome {
                WorthQueryApplicationRetainedMutationOutcome::Committed { retained, .. } => {
                    retained
                }
                WorthQueryApplicationRetainedMutationOutcome::Other(outcome) => {
                    return Err(WorthUiStatusOwnerError::SourceMutationOutcome(Box::new(
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
            Ok(WorthUiStatusPublication {
                value: row.clone(),
                query_receipt: published.receipt().clone(),
                live_close: None,
            })
        })();
        let close = live.close();
        match (result, close) {
            (Ok(publication), close) => Ok(publication.with_live_close(close)),
            (Err(error), WorthQueryApplicationLiveCloseOutcome::Completed(_)) => Err(error),
            (Err(error), close) => Err(error.with_live_close(close)),
        }
    }
}
