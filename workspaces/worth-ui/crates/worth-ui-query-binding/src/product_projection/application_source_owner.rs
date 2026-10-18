use std::num::NonZeroUsize;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome;
use worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt;

use crate::{
    UiApplicationScalarProjectionRegistration, UiProjectionObservation,
    WorthUiScalarProjectionSourceRecord, WorthUiStatusQueryRequest, WorthUiStatusQueryResult,
};

use super::application_authentication::{authenticate, request_scope};
use super::application_runtime::{install_status_application, WorthUiStatusApplication};
use super::status_mutation::{WorthUiStatusMutation, WorthUiStatusMutationResult};
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

    /// Captures one original source command without performing its business effect.
    pub fn prepare_source(
        &self,
        record: WorthUiScalarProjectionSourceRecord,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthUiStatusMutation, WorthUiStatusMutation> {
        WorthUiStatusMutation::source(&self.application, record).prepare(scope)
    }

    /// Attempts one owning retained-commit step, disclosed delivery and native close.
    /// Every stop returns the original continuation for fresh-authorized advancement.
    pub fn publish_source(
        &self,
        record: WorthUiScalarProjectionSourceRecord,
        maximum_history_work: NonZeroUsize,
    ) -> Result<WorthUiStatusPublication, WorthUiStatusMutation> {
        let scope = request_scope();
        let operation = self.prepare_source(record, &scope)?;
        match operation.advance(&scope, maximum_history_work)? {
            WorthUiStatusMutationResult::Source(publication) => Ok(publication),
            WorthUiStatusMutationResult::Action(_) => {
                unreachable!("source construction retains source custody")
            }
        }
    }
}
