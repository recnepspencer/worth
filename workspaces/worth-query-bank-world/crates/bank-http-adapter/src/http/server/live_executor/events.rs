use bank_server::BankAccountActivityLiveOutcome;
use tokio::sync::{mpsc, oneshot};

use super::super::super::protocol::{
    BankHttpAccountActivity, BankHttpAccountActivityEvent, BankHttpDenial, BankHttpDenialKind,
    BankHttpNextAction, BankHttpQueryCapabilityPurpose,
};
use super::super::query_publication::describe_query_publication;

pub(super) fn send_live_event(
    events: &mpsc::Sender<BankHttpAccountActivityEvent>,
    terminal: &mut Option<oneshot::Sender<BankHttpAccountActivityEvent>>,
    event: BankHttpAccountActivityEvent,
    request_id: &str,
) -> bool {
    if is_terminal(&event) {
        send_terminal(terminal, event);
        return false;
    }
    match events.try_send(event) {
        Ok(()) => true,
        Err(mpsc::error::TrySendError::Full(_)) => {
            send_terminal(
                terminal,
                BankHttpAccountActivityEvent::Overflow {
                    request_id: request_id.to_owned(),
                    missed_commit_batches: 1,
                },
            );
            false
        }
        Err(mpsc::error::TrySendError::Closed(_)) => false,
    }
}

pub(super) fn live_event(
    request_id: &str,
    outcome: BankAccountActivityLiveOutcome,
) -> Option<BankHttpAccountActivityEvent> {
    let request_id = request_id.to_owned();
    match outcome {
        BankAccountActivityLiveOutcome::Delivered(update) => {
            Some(BankHttpAccountActivityEvent::Update {
                request_id,
                activity: BankHttpAccountActivity::from(update.result()),
                publication: describe_query_publication(
                    update.receipt(),
                    BankHttpQueryCapabilityPurpose::AccountActivityReview,
                ),
            })
        }
        BankAccountActivityLiveOutcome::Pending => None,
        BankAccountActivityLiveOutcome::Overflow(overflow) => {
            Some(BankHttpAccountActivityEvent::Overflow {
                request_id,
                missed_commit_batches: overflow.missed_commit_batches(),
            })
        }
        BankAccountActivityLiveOutcome::AuthorizationDenied(_)
        | BankAccountActivityLiveOutcome::ProjectionDenied(_)
        | BankAccountActivityLiveOutcome::CauseDenied(_) => Some(denied(
            request_id,
            BankHttpDenial::new(
                BankHttpDenialKind::PermissionDenied,
                BankHttpNextAction::None,
            ),
        )),
        BankAccountActivityLiveOutcome::StalePrincipal
        | BankAccountActivityLiveOutcome::StaleScope => Some(denied(
            request_id,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        )),
        BankAccountActivityLiveOutcome::Cancelled => {
            Some(BankHttpAccountActivityEvent::Cancelled { request_id })
        }
        BankAccountActivityLiveOutcome::DeadlineExceeded => {
            Some(BankHttpAccountActivityEvent::DeadlineExceeded { request_id })
        }
        BankAccountActivityLiveOutcome::Closed => {
            Some(BankHttpAccountActivityEvent::Closed { request_id })
        }
        BankAccountActivityLiveOutcome::Unavailable => {
            Some(BankHttpAccountActivityEvent::Unavailable { request_id })
        }
    }
}

const fn is_terminal(event: &BankHttpAccountActivityEvent) -> bool {
    !matches!(
        event,
        BankHttpAccountActivityEvent::Opened { .. } | BankHttpAccountActivityEvent::Update { .. }
    )
}

pub(super) fn denied(request_id: String, denial: BankHttpDenial) -> BankHttpAccountActivityEvent {
    BankHttpAccountActivityEvent::Denied { request_id, denial }
}

pub(super) fn malformed(request_id: String) -> BankHttpAccountActivityEvent {
    denied(
        request_id,
        BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
    )
}

pub(super) fn send_terminal(
    terminal: &mut Option<oneshot::Sender<BankHttpAccountActivityEvent>>,
    event: BankHttpAccountActivityEvent,
) {
    if let Some(terminal) = terminal.take() {
        let _ = terminal.send(event);
    }
}
