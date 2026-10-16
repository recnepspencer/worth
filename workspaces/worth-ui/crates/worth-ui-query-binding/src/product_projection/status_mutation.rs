//! Ordinary status mutation custody across fresh requests and live disclosure.
use super::application_runtime::WorthUiStatusApplication;
use crate::{
    WorthUiScalarProjectionSourceRecord, WorthUiStatusActionMutationOutcome,
    WorthUiStatusActionOutcome, WorthUiStatusActionRequest, WorthUiStatusLiveDeliveryStop,
    WorthUiStatusMutationOutcome, WorthUiStatusOwnerError, WorthUiStatusPublication,
    WorthUiStatusQueryRequest,
    declaration::{WorthUiApplicationSchema, WorthUiStatusQuery},
};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationLiveNextDenial, WorthQueryApplicationLiveSubscription,
        WorthQueryApplicationReadObservation, WorthQueryApplicationReadObservationDenial,
        WorthQueryApplicationRequestQueryDenial, WorthQueryClosedOperation,
        WorthQueryOperationCloseDenial, WorthQueryOperationConstructionDenial,
        WorthQueryOperationInputCaptureRetryDenial, WorthQuerySourceOnlyDenial,
    },
    primary_graph::{WorthQueryApplicationLiveCloseOutcome, WorthQueryObservedSource},
};
mod native;
use native::NativeCustody;
mod advancement;
mod close;
mod delivery;
mod preparation;

/// One authentic ordinary step stopped; the accompanying continuation retains custody.
#[derive(Debug)]
pub enum WorthUiStatusMutationStop {
    Setup(WorthUiStatusOwnerError),
    Capture(WorthQueryOperationInputCaptureRetryDenial),
    Construction(WorthQueryOperationConstructionDenial),
    Progress,
    Advance(WorthQuerySourceOnlyDenial),
    Terminal,
    RetainedObservationUnavailable,
    PublicationPending,
    CurrentRead(WorthQueryApplicationReadObservationDenial),
    SnapshotQuery(WorthQueryApplicationRequestQueryDenial),
    LiveNext(WorthQueryApplicationLiveNextDenial),
    LiveDelivery(WorthUiStatusLiveDeliveryStop),
    PublicationMismatch,
    Close(WorthQueryOperationCloseDenial),
}

#[derive(Debug)]
pub enum WorthUiStatusMutationResult {
    Source(WorthUiStatusPublication),
    Action(WorthUiStatusActionOutcome),
}

/// Borrowed native terminal evidence grants no retry authority.
pub enum WorthUiStatusTerminalOutcome<'evidence> {
    Source(&'evidence WorthUiStatusMutationOutcome),
    Action(&'evidence WorthUiStatusActionMutationOutcome),
}

/// Actual close receipts; closing original custody does not claim source completion.
pub struct WorthUiStatusMutationClose {
    pub operation: Option<WorthQueryClosedOperation>,
    pub live: Option<WorthQueryApplicationLiveCloseOutcome>,
}

pub(super) enum MutationContext {
    Source,
    Action {
        source_revision: u64,
        status: String,
    },
}

/// Retains the actual Query operation and an authentic installed-runtime keepalive.
/// Requests and live observers are freshly admitted for each advance. Dropping this
/// value abandons its native custody; it does not discharge cleanup.
#[must_use = "retain this continuation until an authenticated advance or lawful close succeeds"]
pub struct WorthUiStatusMutation {
    state: MutationState,
    application: WorthUiStatusApplication,
}

struct MutationState {
    native: Option<NativeCustody>,
    source: Option<WorthQueryObservedSource<WorthUiStatusQuery>>,
    active_revision: Option<u64>,
    context: MutationContext,
    publication: Option<WorthUiStatusPublication>,
    live_close: Option<WorthQueryApplicationLiveCloseOutcome>,
    stop: Option<WorthUiStatusMutationStop>,
}

impl WorthUiStatusMutation {
    pub(super) fn source(
        application: &WorthUiStatusApplication,
        record: WorthUiScalarProjectionSourceRecord,
    ) -> Self {
        Self::new(
            application,
            NativeCustody::source(record),
            MutationContext::Source,
        )
    }
    pub(super) fn action(
        application: &WorthUiStatusApplication,
        action: WorthUiStatusActionRequest,
    ) -> Self {
        let context = MutationContext::Action {
            source_revision: action.source_revision(),
            status: action.status().to_owned(),
        };
        Self::new(application, NativeCustody::action(action), context)
    }
    fn new(
        application: &WorthUiStatusApplication,
        native: NativeCustody,
        context: MutationContext,
    ) -> Self {
        Self {
            state: MutationState {
                native: Some(native),
                source: None,
                active_revision: None,
                context,
                publication: None,
                live_close: None,
                stop: None,
            },
            application: application.clone(),
        }
    }
    pub fn stop(&self) -> Option<&WorthUiStatusMutationStop> {
        self.state.stop.as_ref()
    }
    pub fn terminal_outcome(&self) -> Option<WorthUiStatusTerminalOutcome<'_>> {
        self.state
            .native
            .as_ref()
            .and_then(NativeCustody::terminal_outcome)
    }
    pub fn capture_denial(
        &self,
    ) -> Option<&worth_query_host::facade::application_entry::WorthQueryOperationInputCaptureDenial>
    {
        self.state
            .native
            .as_ref()
            .and_then(NativeCustody::capture_denial)
    }
    pub fn native_admission_denial(
        &self,
    ) -> Option<
        &worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
    > {
        self.state
            .native
            .as_ref()
            .and_then(NativeCustody::admission_denial)
    }
    pub fn commit_outcome(
        &self,
    ) -> Option<&worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted> {
        match self.terminal_outcome()? {
            WorthUiStatusTerminalOutcome::Source(outcome) => outcome.commit_outcome(),
            WorthUiStatusTerminalOutcome::Action(outcome) => outcome.commit_outcome(),
        }
    }
    /// Actual most recent observer close outcome, including an unsuccessful release.
    pub fn live_close(&self) -> Option<&WorthQueryApplicationLiveCloseOutcome> {
        self.state.live_close.as_ref()
    }
    pub fn take_live_close(&mut self) -> Option<WorthQueryApplicationLiveCloseOutcome> {
        self.state.live_close.take()
    }
    fn refused(mut self, stop: WorthUiStatusMutationStop) -> Self {
        self.state.stop = Some(stop);
        self
    }
}
impl std::fmt::Debug for WorthUiStatusMutation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthUiStatusMutation")
            .field("stop", &self.state.stop)
            .finish_non_exhaustive()
    }
}
#[cfg(test)]
mod tests;
