//! Separate callback route: bounded bytes, fixed verifier, Query custody, ACK.

use std::time::Instant;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryInboundAdmissionDenial;

use crate::http::protocol::inbound_completion::MAXIMUM_COMPLETION_BYTES;
use crate::http::server::routes::BankHttpRouteState;

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
    if envelope.len() > MAXIMUM_COMPLETION_BYTES {
        return (StatusCode::PAYLOAD_TOO_LARGE, Vec::new());
    }
    let Ok(slot) = state.rail_slots.try_acquire_owned() else {
        return (StatusCode::TOO_MANY_REQUESTS, Vec::new());
    };
    let cancellation = WorthQueryCancellationSource::new();
    let _cancel_on_drop = CancelOnDrop(cancellation.clone());
    let deadline = Instant::now() + state.maximum_deadline;
    let wake = state.rail_maintenance_wake;
    let task = tokio::task::spawn_blocking(move || {
        let _slot = slot;
        let request = WorthQueryRequestScope::new(deadline, cancellation.token());
        let result = route.receive_and_sign(&envelope, &request);
        if matches!(
            &result,
            Ok((_, true))
                | Err(WorthQueryInboundAdmissionDenial::RetryBeforeAcceptance
                    | WorthQueryInboundAdmissionDenial::PublicationRetryRequired
                    | WorthQueryInboundAdmissionDenial::RecoveryUnavailable
                    | WorthQueryInboundAdmissionDenial::RecoveryStaleProduct
                    | WorthQueryInboundAdmissionDenial::TerminalCleanupUnavailable)
        ) {
            wake.notify_one();
        }
        result
    });
    match tokio::time::timeout(state.maximum_deadline, task).await {
        Ok(Ok(Ok((ack, _)))) => (StatusCode::OK, ack),
        Ok(Ok(Err(denial))) => (status(denial), Vec::new()),
        Ok(Err(_)) | Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Vec::new()),
    }
}

fn status(denial: WorthQueryInboundAdmissionDenial) -> StatusCode {
    use WorthQueryInboundAdmissionDenial as Denial;
    match denial {
        Denial::Oversized => StatusCode::PAYLOAD_TOO_LARGE,
        Denial::Verification(_) | Denial::Expired => StatusCode::UNAUTHORIZED,
        Denial::CapacityExhausted => StatusCode::TOO_MANY_REQUESTS,
        Denial::RetryBeforeAcceptance
        | Denial::TimeUnavailable
        | Denial::TerminalCleanupUnavailable
        | Denial::PublicationInProgress
        | Denial::PublicationRetryRequired
        | Denial::RecoveryStaleProduct
        | Denial::RecoveryUnavailable
        | Denial::SourceRevoked
        | Denial::OwnerReadDenied(_) => StatusCode::SERVICE_UNAVAILABLE,
        Denial::ForeignVerifier
        | Denial::ForeignOwner
        | Denial::UnsupportedOutbox
        | Denial::UnknownCorrelation
        | Denial::IncompatibleMeaning
        | Denial::SourceRetired
        | Denial::MessageIdentityConflict
        | Denial::CorrelationAlreadyOwned => StatusCode::CONFLICT,
    }
}
