//! Source custody stops, declined evidence and non-committed correction outcomes.
use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as Keyed;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryTerminalSourceEvidence,
};

use crate::application_home::{
    WorthQueryHomeAuthoredCorrectionOutcome, WorthQueryHomeCorrectionOutcome,
    WorthQueryHomeOwnedHistoryDenial,
};
use crate::source_custody::{WorthQueryAnyTerminalSourceEvidence, WorthQuerySourceCustodyPending};

use super::request::{ALREADY_DONE, BUSY, CANCELLED, RULE_REFUSED, TIMED_OUT};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const STILL_SAVING: Summary = Summary::with_message(
    Category::Unavailable,
    "The change is still being saved. Check again in a moment before retrying.",
);
const NOT_APPLIED: Summary = Summary::with_message(
    Category::Unavailable,
    "The earlier attempt didn't finish, so nothing was changed. Try again.",
);
const DIFFERENT_REQUEST: Summary = Summary::with_message(
    Category::Conflict,
    "A different change was already saved for this request. Refresh and try again.",
);

/// A mutation outcome that did not commit freshly, or `None` when it did.
pub(super) fn uncommitted_outcome<Denial, Result>(
    outcome: &WorthQueryApplicationMutationOutcome<Denial, Result>,
) -> Option<Summary> {
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { .. } => None,
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
        | WorthQueryApplicationMutationOutcome::RecoveredCommitted(_) => Some(ALREADY_DONE),
        WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift => Some(DIFFERENT_REQUEST),
        WorthQueryApplicationMutationOutcome::DomainDenied(_) => Some(RULE_REFUSED),
        WorthQueryApplicationMutationOutcome::Cancelled => Some(CANCELLED),
        WorthQueryApplicationMutationOutcome::DeadlineExceeded => Some(TIMED_OUT),
        WorthQueryApplicationMutationOutcome::Commit(cause) => Some(cause.user_summary()),
    }
}

/// Stop evidence of a declined source command (its handler, admission or keyed settlement).
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> WorthQueryUserFacingRefusal
    for WorthQueryTerminalSourceEvidence<S, I>
{
    fn user_summary(&self) -> Summary {
        match self {
            Self::Outcome(outcome) => uncommitted_outcome(outcome).unwrap_or(ALREADY_DONE),
            Self::AdmissionStopped(cause) => cause.user_summary(),
            Self::Keyed(resolution) => keyed(resolution),
        }
    }
}

/// A keyed retry settled from durable idempotency instead of a live outcome.
fn keyed(resolution: &Keyed) -> Summary {
    match resolution {
        Keyed::RecoveredCommitted(_) => ALREADY_DONE,
        Keyed::IntentDrift => DIFFERENT_REQUEST,
        Keyed::ProductUnpublished(_) | Keyed::ProductPreparationPending(_) => STILL_SAVING,
        Keyed::ReleasedOriginal(_) | Keyed::Unseen => NOT_APPLIED,
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryAnyTerminalSourceEvidence {
    fn user_summary(&self) -> Summary {
        self.summary().clone()
    }
}

impl WorthQueryUserFacingRefusal for WorthQuerySourceCustodyPending {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Retention(_) | Self::Progress | Self::Close(_) => STILL_SAVING,
            Self::Refused(_) => BUSY,
        }
    }
}

impl<Schema, Program> WorthQueryHomeAuthoredCorrectionOutcome<Schema, Program> {
    /// Why this undo or redo did not commit, for an end user; `None` once committed.
    pub fn refusal_summary(&self) -> Option<Summary> {
        match self {
            Self::Committed { .. } => None,
            Self::Declined { evidence, .. } => Some(evidence.user_summary()),
            Self::Pending { cause, .. } => Some(cause.user_summary()),
            Self::IdempotencyIntentDrift => Some(DIFFERENT_REQUEST),
            Self::PublicationPending | Self::ProductUnpublished(_) => Some(STILL_SAVING),
        }
    }
}

impl WorthQueryHomeCorrectionOutcome {
    /// Why this undo or redo did not commit, for an end user; `None` once committed.
    pub fn refusal_summary(&self) -> Option<Summary> {
        match self {
            Self::Committed { .. } | Self::AlreadyCommitted(_) | Self::RecoveredCommitted(_) => {
                None
            }
            Self::DomainDenied(_) => Some(RULE_REFUSED),
            Self::Other(outcome) => uncommitted_outcome(outcome),
            Self::PublicationPending | Self::ProductUnpublished(_) => Some(STILL_SAVING),
        }
    }
}

impl<Schema> WorthQueryUserFacingRefusal for WorthQueryHomeOwnedHistoryDenial<'_, Schema> {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Fork(cause) => cause.user_summary(),
            Self::Advancement { denial, .. } | Self::AdvancementAfterConsume(denial) => {
                denial.user_summary()
            }
            Self::Attachment { .. } => BUSY,
        }
    }
}
