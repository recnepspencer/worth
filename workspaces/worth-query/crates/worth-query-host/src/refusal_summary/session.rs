//! Session actions, live views, managed-index watches, catch-up and sharing.
use crate::session::{
    WorthQueryLocalSessionDenial, WorthQueryOwnedSessionDenial, WorthQuerySessionRunDenial,
    WorthQuerySessionSnapshotDenial, WorthQueryTypedLiveCloseDenial,
    WorthQueryTypedLiveRefreshDenial, WorthQueryTypedManagedIndexRefreshDenial,
};
use crate::shared_access::{
    AuthenticationRefusalReason, AuthorizationRefusalReason, ProtocolRefusalReason,
    SharedAccessDenial, WorthQueryScopeGrantRefusal,
};
use crate::temporal_contribution::WorthQueryTemporalCatchUpDenial;

use super::request::{
    ALREADY_DONE, BUSY, CANCELLED, CAPACITY, INTERNAL, MISSING_ITEM, PERMISSION, SESSION_EXPIRED,
    TIMED_OUT, UNSUPPORTED,
};
use super::stops::{authorization, projection, session};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const LIVE_BEHIND: Summary = Summary::with_message(
    Category::Unavailable,
    "This view fell behind recent changes. Reopen it to see the latest data.",
);
const LIVE_CLOSED: Summary = Summary::with_message(
    Category::Unavailable,
    "This view was closed. Reopen it to keep watching for changes.",
);
const SIGN_IN_AGAIN: Summary = Summary::with_message(
    Category::PermissionDenied,
    "Your access could not be confirmed. Sign in again and retry.",
);
const SHARE_UNREACHABLE: Summary = Summary::with_message(
    Category::Unavailable,
    "The shared workspace can't be reached right now. Try again shortly.",
);

/// A domain denial renders through the application's own implementation.
impl<DomainDenial: WorthQueryUserFacingRefusal> WorthQueryUserFacingRefusal
    for WorthQuerySessionRunDenial<DomainDenial>
{
    fn user_summary(&self) -> Summary {
        match self {
            Self::Request(cause) => cause.user_summary(),
            Self::DomainDenied(denial) => denial.user_summary(),
            Self::AlreadyCommitted(_) | Self::RecoveredCommitted(_) => ALREADY_DONE,
            Self::IdempotencyIntentDrift => Summary::with_message(
                Category::Conflict,
                "A different change was already saved for this request. Refresh and try again.",
            ),
            Self::AutoIdempotencyKeysExhausted => Summary::with_message(
                Category::Conflict,
                "This change was retried too many times. Refresh and try again.",
            ),
            Self::Cancelled => CANCELLED,
            Self::DeadlineExceeded => TIMED_OUT,
            Self::Uncommitted(cause) => cause.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for std::convert::Infallible {
    fn user_summary(&self) -> Summary {
        match *self {}
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryTypedLiveRefreshDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::AuthorizationDenied(denial) => authorization(&denial.kind()),
            Self::StalePrincipal => PERMISSION,
            Self::ProjectionDenied(kind) => projection(kind),
            Self::StaleScope => MISSING_ITEM,
            Self::Overflow(_) => LIVE_BEHIND,
            Self::ResultLimitExceeded => Summary::with_message(
                Category::Capacity,
                "There are too many results to show live. Narrow the view and try again.",
            ),
            Self::ExecutionRequest(kind) => session(kind),
            Self::Next(_) | Self::Unavailable => BUSY,
            Self::Cancelled => CANCELLED,
            Self::DeadlineExceeded => TIMED_OUT,
            Self::Closed => LIVE_CLOSED,
            Self::SessionExpired => SESSION_EXPIRED,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryTypedLiveCloseDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Unavailable => BUSY,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryTypedManagedIndexRefreshDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Query(cause) => cause.user_summary(),
            Self::Overflow { .. } => LIVE_BEHIND,
            Self::Unavailable => BUSY,
            Self::Closed => LIVE_CLOSED,
        }
    }
}

/// A closed Home ends its owned sessions; other refusals keep their summary.
impl<Denial: WorthQueryUserFacingRefusal> WorthQueryUserFacingRefusal
    for WorthQueryOwnedSessionDenial<Denial>
{
    fn user_summary(&self) -> Summary {
        match self {
            Self::HomeClosed => Summary::with_message(
                Category::NotFound,
                "This data has been closed. Open it again to continue.",
            ),
            Self::Denied(denial) => denial.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryLocalSessionDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Admission(cause) => cause.user_summary(),
            Self::Authentication(cause) => cause.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQuerySessionSnapshotDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::ProductSelection(_) => BUSY,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryTemporalCatchUpDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Advancement(cause) => cause.user_summary(),
            Self::BranchSelection(_) | Self::ClockPort(_) => BUSY,
            Self::TemporalBindingNotInstalled => UNSUPPORTED,
            Self::InvalidMaxSteps { .. } | Self::ForeignClockHandle | Self::ClockAdvance(_) => {
                INTERNAL
            }
        }
    }
}

impl WorthQueryUserFacingRefusal for SharedAccessDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::AuthenticationRefused(AuthenticationRefusalReason::StaleIncarnation) => {
                SHARE_UNREACHABLE
            }
            Self::AuthenticationRefused(_) => SIGN_IN_AGAIN,
            Self::AuthorizationRefused(AuthorizationRefusalReason::ScopeNotFound) => MISSING_ITEM,
            Self::AuthorizationRefused(_) | Self::GrantMutationRefused(_) => PERMISSION,
            Self::ProtocolRefused(ProtocolRefusalReason::UnsupportedVersion) => {
                Summary::with_message(
                    Category::Unsupported,
                    "The other app uses a version this app can't talk to. Update both and retry.",
                )
            }
            Self::ProtocolRefused(_) => Summary::with_message(
                Category::Unsupported,
                "The other app sent something this app couldn't understand.",
            ),
            Self::BudgetRefused(_) => CAPACITY,
            Self::WatchRefused(_) => LIVE_CLOSED,
            Self::DirectoryHomeRequired => Summary::with_message(
                Category::Unsupported,
                "Sharing needs data saved on this device, not a temporary workspace.",
            ),
            Self::NonLoopbackAddressRefused => Summary::with_message(
                Category::PermissionDenied,
                "Sharing is only allowed with apps on this computer.",
            ),
            Self::InvalidGrantIdentifier => {
                Summary::with_message(Category::Validation, "That sharing invitation isn't valid.")
            }
            Self::InvalidAuthConfiguration => INTERNAL,
            Self::HostUnavailable | Self::TransportIo(_) => SHARE_UNREACHABLE,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryScopeGrantRefusal {
    fn user_summary(&self) -> Summary {
        match self {
            Self::GranteeMappingDisabled | Self::GranteeUnmapped => Summary::with_message(
                Category::NotFound,
                "That person isn't set up to receive access yet.",
            ),
            Self::InvalidGrantKey => {
                Summary::with_message(Category::Validation, "That sharing invitation isn't valid.")
            }
        }
    }
}
