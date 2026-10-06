//! Separate callback route: bounded bytes, fixed verifier, Query custody, ACK.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryCommittedDispatchOutboxReadDenial, WorthQueryInboundAdmissionDenial,
};

use crate::http::protocol::inbound_completion::MAXIMUM_COMPLETION_BYTES;
use crate::http::server::routes::BankHttpRouteState;

use super::route::BankRailCompletionRoute;

#[derive(Clone)]
pub(in crate::http::server) struct BankRailCompletionEndpointState {
    pub route: Arc<dyn BankRailCompletionRoute>,
    pub slots: Arc<tokio::sync::Semaphore>,
    pub wake: Arc<tokio::sync::Notify>,
    pub maximum_deadline: Duration,
}

struct CancelOnDrop(WorthQueryCancellationSource);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

pub(in crate::http::server) async fn rail_completion(
    State(state): State<BankHttpRouteState>,
    envelope: Bytes,
) -> (StatusCode, Vec<u8>) {
    let Some(route) = state.rail else {
        return (StatusCode::NOT_FOUND, Vec::new());
    };
    receive(
        BankRailCompletionEndpointState {
            route,
            slots: state.rail_slots,
            wake: state.rail_maintenance_wake,
            maximum_deadline: state.maximum_deadline,
        },
        envelope,
    )
    .await
}

pub(in crate::http::server) async fn receive(
    state: BankRailCompletionEndpointState,
    envelope: Bytes,
) -> (StatusCode, Vec<u8>) {
    if envelope.len() > MAXIMUM_COMPLETION_BYTES {
        return (StatusCode::PAYLOAD_TOO_LARGE, Vec::new());
    }
    let Ok(slot) = state.slots.try_acquire_owned() else {
        return (StatusCode::TOO_MANY_REQUESTS, Vec::new());
    };
    let cancellation = WorthQueryCancellationSource::new();
    let _cancel_on_drop = CancelOnDrop(cancellation.clone());
    let deadline = Instant::now() + state.maximum_deadline;
    let wake = state.wake;
    let route = state.route;
    let task = tokio::task::spawn_blocking(move || {
        let _slot = slot;
        let request = WorthQueryRequestScope::new(deadline, cancellation.token());
        let result = route.receive_and_sign(&envelope, &request);
        if result
            .as_ref()
            .is_ok_and(|outcome| outcome.requires_maintenance())
            || matches!(
                &result,
                Err(WorthQueryInboundAdmissionDenial::RetryBeforeAcceptance
                    | WorthQueryInboundAdmissionDenial::PublicationRetryRequired
                    | WorthQueryInboundAdmissionDenial::RecoveryUnavailable
                    | WorthQueryInboundAdmissionDenial::RecoveryStaleProduct
                    | WorthQueryInboundAdmissionDenial::TerminalCleanupUnavailable)
            )
        {
            wake.notify_one();
        }
        result
    });
    match tokio::time::timeout(state.maximum_deadline, task).await {
        Ok(Ok(Ok(outcome))) => (StatusCode::OK, outcome.into_ack()),
        Ok(Ok(Err(denial))) => (status(denial), Vec::new()),
        Ok(Err(_)) | Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Vec::new()),
    }
}

fn status(denial: WorthQueryInboundAdmissionDenial) -> StatusCode {
    use WorthQueryInboundAdmissionDenial as Denial;
    match denial {
        Denial::Oversized => StatusCode::PAYLOAD_TOO_LARGE,
        Denial::Verification(_) | Denial::Expired | Denial::ValidityWindowExceeded => {
            StatusCode::UNAUTHORIZED
        }
        Denial::CapacityExhausted => StatusCode::TOO_MANY_REQUESTS,
        Denial::RetryBeforeAcceptance
        | Denial::TimeUnavailable
        | Denial::TerminalCleanupUnavailable
        | Denial::PublicationInProgress
        | Denial::PublicationRetryRequired
        | Denial::RecoveryStaleProduct
        | Denial::RecoveryUnavailable
        | Denial::SourceRevoked => StatusCode::SERVICE_UNAVAILABLE,
        Denial::OwnerReadDenied(read) => owner_read_status(read),
        Denial::ForeignVerifier
        | Denial::ForeignOwner
        | Denial::AuthenticatedPermanent(_)
        | Denial::OriginalDispatchHasNoInboundSupport
        | Denial::UnsupportedOutbox
        | Denial::UnknownCorrelation
        | Denial::IncompatibleMeaning
        | Denial::SourceRetired
        | Denial::MessageIdentityConflict
        | Denial::CorrelationAlreadyOwned => StatusCode::CONFLICT,
    }
}

/// The sender retries only a read that can later succeed. The exact committed
/// basis of an owner is gone for good, so that callback is refused as gone;
/// spent identities and owner faults are the server's to fix.
fn owner_read_status(read: WorthQueryCommittedDispatchOutboxReadDenial) -> StatusCode {
    use WorthQueryCommittedDispatchOutboxReadDenial as Read;
    match read {
        Read::ExactCommitUnavailable => StatusCode::GONE,
        Read::PendingPublication
        | Read::CommittedIndexUnavailable
        | Read::ActiveSnapshotCapacityExhausted { .. } => StatusCode::SERVICE_UNAVAILABLE,
        Read::Missing => StatusCode::CONFLICT,
        Read::SnapshotIdentityExhausted
        | Read::ForeignRuntime
        | Read::AmbiguousCorrelation
        | Read::WrongRecordKind
        | Read::NotAuthoritative
        | Read::Malformed
        | Read::CommitMismatch
        | Read::RecordMismatch => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
#[path = "endpoint/owner_read_status_tests.rs"]
mod owner_read_status_tests;
