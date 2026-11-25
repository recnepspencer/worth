//! Local setup refusals: build identity, Home limits, the local time zone, and
//! local-process authentication and adapter admission.
use worth_foundational::facade::InvalidApplicationBuildIdentity;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticationAdapterAdmissionDenial as AdapterAdmission,
    WorthQueryAuthenticationAdapterFailureKind as AdapterFailure, WorthQueryAuthenticationDenial,
    WorthQueryAuthenticationDenialKind as Authentication,
    WorthQueryValidatedExternalPrincipalDenial as Principal,
};
use worth_runtime_world::facade::RuntimeWorldBudgetDenial;

use crate::local_calendar::WorthQueryLocalCalendarDenial;

use super::request::{CANCELLED, TIMED_OUT, VERSION_MISMATCH};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const SIGN_IN_AGAIN: Summary = Summary::with_message(
    Category::PermissionDenied,
    "Your access could not be confirmed. Sign in again and retry.",
);
const CLOCK_WRONG: Summary = Summary::with_message(
    Category::PermissionDenied,
    "Your sign-in isn't valid at this time. Check this computer's clock and sign in again.",
);
const SETUP_DEFECT: Summary = Summary::with_message(
    Category::Internal,
    "The app's sign-in setup is invalid. Please report this problem.",
);

impl WorthQueryUserFacingRefusal for InvalidApplicationBuildIdentity {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Missing | Self::Malformed | Self::Zero => Summary::with_message(
                Category::Internal,
                "This copy of the app has no valid build identity. Reinstall or report it.",
            ),
        }
    }
}

/// The configured Home limits (`WorthQueryApplicationLimits::install`) refused a bound.
impl WorthQueryUserFacingRefusal for RuntimeWorldBudgetDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ZeroLimit { .. } | Self::SizeOverflow { .. } => Summary::with_message(
                Category::Internal,
                "The app's storage limits are set up incorrectly. Please report this problem.",
            ),
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryLocalCalendarDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::SystemZoneUnreadable(_) | Self::MalformedZone => Summary::with_message(
                Category::Storage,
                "This computer's time zone couldn't be read. Check its date and time settings.",
            ),
            Self::OffsetOutOfRange | Self::InstantOutOfRange => Summary::with_message(
                Category::Validation,
                "That date or time is outside the range the app can show.",
            ),
        }
    }
}

/// Local-process (and every adapter's) authentication.
impl WorthQueryUserFacingRefusal for WorthQueryAuthenticationDenial {
    fn user_summary(&self) -> Summary {
        match self.kind() {
            Authentication::Cancelled => CANCELLED,
            Authentication::DeadlineExceeded => TIMED_OUT,
            Authentication::Expired => Summary::with_message(
                Category::PermissionDenied,
                "Your sign-in expired. Sign in again and retry.",
            ),
            Authentication::ValidationTimeInFuture => CLOCK_WRONG,
            Authentication::AudienceMismatch | Authentication::MethodMismatch => SIGN_IN_AGAIN,
            Authentication::ForeignClock => SETUP_DEFECT,
            Authentication::AdapterFailed(failure) => adapter_failure(failure),
        }
    }
}

fn adapter_failure(failure: AdapterFailure) -> Summary {
    match failure {
        AdapterFailure::CredentialRejected
        | AdapterFailure::CredentialRevoked
        | AdapterFailure::BindingMismatch => SIGN_IN_AGAIN,
        AdapterFailure::CredentialExpired => Summary::with_message(
            Category::PermissionDenied,
            "Your sign-in expired. Sign in again and retry.",
        ),
        AdapterFailure::DependencyUnavailable => Summary::with_message(
            Category::Unavailable,
            "Sign-in can't be checked right now. Try again in a moment.",
        ),
        AdapterFailure::Cancelled => CANCELLED,
        AdapterFailure::DeadlineExceeded => TIMED_OUT,
        AdapterFailure::ProtocolViolation => VERSION_MISMATCH,
    }
}

/// Admission of an authentication adapter, including the local-process one.
impl WorthQueryUserFacingRefusal for AdapterAdmission {
    fn user_summary(&self) -> Summary {
        match self {
            Self::InvalidConfigurationIdentity
            | Self::CanonicalEntryBudgetExceeded
            | Self::CanonicalEncodedByteBudgetExceeded
            | Self::CanonicalDigestSlotRejected => SETUP_DEFECT,
        }
    }
}

/// A local-process principal or validated external principal could not be formed.
impl WorthQueryUserFacingRefusal for Principal {
    fn user_summary(&self) -> Summary {
        match self {
            Self::InvalidTimeRange => CLOCK_WRONG,
            Self::InvalidAttribute | Self::TooManyAttributes | Self::AttributesTooLarge => {
                SETUP_DEFECT
            }
        }
    }
}
