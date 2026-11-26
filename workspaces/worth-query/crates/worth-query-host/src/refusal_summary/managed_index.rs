//! Managed-index refusals: watch close, rebuild and readiness.
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexDenial, WorthQueryManagedIndexDenialCause as Cause,
    WorthQueryManagedIndexRefusal as Refusal,
};

use super::live_open::live_open;
use super::request::{
    BUSY, CANCELLED, CAPACITY, INTERNAL, STALE, STORAGE, TIMED_OUT, VERSION_MISMATCH,
};
use super::stops::{native, principal_resolution, work};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const INDEX_PREPARING: Summary = Summary::with_message(
    Category::Unavailable,
    "Search is still being prepared. Try again in a moment.",
);
const INDEX_CLOSED: Summary = Summary::with_message(
    Category::Unavailable,
    "This search was closed. Reopen it to keep watching for changes.",
);

impl WorthQueryUserFacingRefusal for WorthQueryManagedIndexDenial {
    fn user_summary(&self) -> Summary {
        match self.cause() {
            Cause::Refused(refusal) => refused(refusal),
            Cause::Execution(cause) => native(cause),
            Cause::Request(cause) => work(&cause),
            Cause::Interrupted(Interruption::Cancelled) => CANCELLED,
            Cause::Interrupted(Interruption::DeadlineExceeded) => TIMED_OUT,
            Cause::Scope(cause) => cause.user_summary(),
            Cause::QueryAdmission(cause) => cause.user_summary(),
            Cause::QueryRead(cause) => cause.user_summary(),
            Cause::LiveAdmission(cause) => live_open(cause),
            Cause::Principal(denial) => principal_resolution(denial.kind()),
            Cause::SourceVerification { .. } | Cause::Source(_) => STALE,
            Cause::CurrentControl(_) | Cause::Product(_) | Cause::NativeAdmission(_) => BUSY,
            Cause::GraphWork(_) => CAPACITY,
            Cause::Work(_) | Cause::NativeSource(_) => STORAGE,
            Cause::Installation(_) => INTERNAL,
        }
    }
}

fn refused(refusal: Refusal) -> Summary {
    match refusal {
        Refusal::BuildInProgress
        | Refusal::NotReady
        | Refusal::StaleImage
        | Refusal::RootUnavailable
        | Refusal::SnapshotUnavailable => INDEX_PREPARING,
        Refusal::Disposed => INDEX_CLOSED,
        Refusal::IndexNotInstalled(_) => VERSION_MISMATCH,
        Refusal::EpochExhausted
        | Refusal::RegistrationCapacityExceeded
        | Refusal::RegistrationIdentityExhausted
        | Refusal::SessionIdentityExhausted
        | Refusal::ManagedRunIdentityExhausted
        | Refusal::InstallationAccountingOverflow
        | Refusal::CapacityAccountingOverflow => CAPACITY,
        Refusal::ForeignPhase
        | Refusal::ForeignBinding
        | Refusal::UnsupportedMode
        | Refusal::NotIndexParent
        | Refusal::MissingProvider
        | Refusal::MissingProbe
        | Refusal::ForeignAssociation
        | Refusal::SessionAuthorityMismatch
        | Refusal::ForeignSelection
        | Refusal::ForeignContext
        | Refusal::InvariantOwned
        | Refusal::PlanAffinityMismatch
        | Refusal::BasisBranchMismatch
        | Refusal::TerminalReleaseMismatch
        | Refusal::MutationSession
        | Refusal::ForeignBasis
        | Refusal::ForeignGraph
        | Refusal::ForeignReadProof
        | Refusal::ForeignReadIntent
        | Refusal::PriorEffectRead => INTERNAL,
    }
}
