//! Request-level refusals shared by actions, queries and corrections.
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationCommitDenialKind as CommitDenialKind,
        WorthQueryApplicationUncommitted, WorthQueryKeyedEntityOutcome,
        WorthQueryKeyedEntityRefusal, WorthQueryOperationBodyDenial,
    },
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequestMutationDenial, WorthQueryApplicationRequestQueryDenial,
};

use super::stops::{authorization, principal_resolution};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

pub(super) const PERMISSION: Summary = Summary::of(Category::PermissionDenied);
pub(super) const SESSION_EXPIRED: Summary = Summary::of(Category::SessionExpired);
pub(super) const STALE: Summary = Summary::of(Category::Conflict);
pub(super) const INTERNAL: Summary = Summary::of(Category::Internal);
pub(super) const UNSUPPORTED: Summary = Summary::of(Category::Unsupported);
pub(super) const CAPACITY: Summary = Summary::of(Category::Capacity);
pub(super) const BUSY: Summary = Summary::of(Category::Unavailable);
pub(super) const STORAGE: Summary = Summary::of(Category::Storage);
pub(super) const CANCELLED: Summary = Summary::with_message(
    Category::Interrupted,
    "That was cancelled before it finished. Nothing was changed.",
);
pub(super) const TIMED_OUT: Summary = Summary::with_message(
    Category::Interrupted,
    "That took too long and was stopped. Nothing was changed. Try again.",
);
pub(super) const ALREADY_DONE: Summary = Summary::with_message(
    Category::Conflict,
    "That was already done. Refresh to see the result.",
);
pub(super) const MISSING_ITEM: Summary = Summary::with_message(
    Category::NotFound,
    "That item could not be found. It may have been removed.",
);
/// A scope or key matched several entities: the app scoped by a field that
/// is not unique, a declaration defect rather than a user error.
pub(super) const AMBIGUOUS_KEY: Summary = Summary::with_message(
    Category::Internal,
    "More than one item matches that key, so the app can't tell which one you meant. \
     Please report this problem.",
);
pub(super) const ALREADY_EXISTS: Summary = Summary::with_message(
    Category::Conflict,
    "Something with that name or key already exists. Choose a different one.",
);
pub(super) const RULE_REFUSED: Summary = Summary::with_message(
    Category::Validation,
    "That change breaks one of the app's rules, so it wasn't saved.",
);
pub(super) const BAD_INPUT: Summary = Summary::with_message(
    Category::Validation,
    "Some of the values entered aren't valid. Check them and try again.",
);
pub(super) const VERSION_MISMATCH: Summary = Summary::with_message(
    Category::Unsupported,
    "This data needs a different version of the app. Update the app and try again.",
);

impl WorthQueryUserFacingRefusal for WorthQueryAdvancementDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Resource(_) => CAPACITY,
            Self::Interrupted(WorthQueryManagedComputationInterruption::Cancelled) => CANCELLED,
            Self::Interrupted(WorthQueryManagedComputationInterruption::DeadlineExceeded) => {
                TIMED_OUT
            }
            Self::NestedOpening | Self::NestedStopped | Self::Panicked => INTERNAL,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryOperationBodyDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::SourceMismatch { .. } => STALE,
            Self::InputMismatch { .. } => BAD_INPUT,
            Self::RequirementUnmet { .. } | Self::DeclaredRequirementUnmet { .. } => RULE_REFUSED,
            Self::DomainRefused(refusal) => refusal.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationRequestMutationDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::Authorization(denial) => authorization(&denial.kind()),
            Self::PrincipalResolution(denial) => principal_resolution(denial.kind()),
            Self::WorkflowAuthoritySpent => PERMISSION,
            Self::ScopeResolution(cause) => cause.user_summary(),
            Self::Identity(cause) => cause.user_summary(),
            Self::InputCapture(cause) => match (cause.encoding(), cause.execution_stop()) {
                (Some(encoding), _) => encoding.user_summary(),
                (None, Some(stop)) => stop.user_summary(),
                (None, None) => CAPACITY,
            },
            Self::WorkflowCapturedInput(_) => BAD_INPUT,
            Self::Handler(cause) => cause.user_summary(),
            Self::SourceExpectation(_) | Self::WorkflowTransitionCurrentness(_) => STALE,
            Self::Idempotency(_) | Self::PreparationSpent => ALREADY_DONE,
            Self::ProductSelection(_) => BUSY,
            Self::ApplicationProgramRequired
            | Self::ApplicationProgramMismatch
            | Self::ProgramSelection(_)
            | Self::RequiresWorkflowTransition => VERSION_MISMATCH,
            Self::BindingInstallation(_)
            | Self::CapabilityInstallation(_)
            | Self::SignedCreation(_) => INTERNAL,
        }
    }
}

/// Handler stop evidence: the application's own body or its write admission.
impl WorthQueryUserFacingRefusal for MutationHandlerExecutionDenial {
    fn user_summary(&self) -> Summary {
        if let Some(keyed) = self.keyed_entity() {
            return keyed.user_summary();
        }
        match self {
            Self::Handler(_) | Self::Projection(_) => RULE_REFUSED,
            Self::Attempt(_) => STALE,
            Self::InputNotAdmitted => BAD_INPUT,
            Self::DurableBindingAdmission(_)
            | Self::DurableStateAdmission(_)
            | Self::HandlerNotInstalled => VERSION_MISMATCH,
            Self::ForeignAdvancementPhase(_) | Self::WorkflowControl => INTERNAL,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationRequestQueryDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::PrincipalResolution(denial) => principal_resolution(denial.kind()),
            // A missing grant is a permission refusal; a stopped admission keeps
            // its own work, capacity or interruption category.
            Self::CapabilityAdmission(denial) => authorization(&denial.kind()),
            Self::Admission(cause) => cause.user_summary(),
            Self::ScopeResolution(cause) => cause.user_summary(),
            Self::InputPreparation(cause) => cause.user_summary(),
            Self::Limit(_) => Summary::with_message(
                Category::Capacity,
                "There are too many results to show. Narrow the search and try again.",
            ),
            Self::ProductSelection(_) | Self::OutputSettlement(_) => BUSY,
            Self::ManagedIndex(cause) => cause.user_summary(),
            Self::Execution(cause) => cause.user_summary(),
            Self::ContinuationExecution(cause) => cause.user_summary(),
            Self::RequestMode | Self::BindingInstallation(_) | Self::CapabilityInstallation(_) => {
                INTERNAL
            }
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationUncommitted {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ProductStale(_) | Self::Stale(_) => STALE,
            Self::Cancelled => CANCELLED,
            Self::TimedOut => TIMED_OUT,
            Self::Denied(denial) if denial.kind() == CommitDenialKind::UniqueValueTaken => {
                ALREADY_EXISTS
            }
            Self::Denied(denial) => denial
                .domain_refusal()
                .map_or(RULE_REFUSED, Summary::domain),
            Self::NoEffect(_) | Self::Aborted => RULE_REFUSED,
            Self::OutputAdopted(_) => ALREADY_DONE,
            Self::CleanupRequired(_)
            | Self::ProductUnpublished(_)
            | Self::ProductPreparationPending(_)
            | Self::Deferred(_)
            | Self::Indeterminate(_) => Summary::with_message(
                Category::Unavailable,
                "The change is still being saved. Check again in a moment before retrying.",
            ),
            Self::ProductPreparationRefused(_) => STORAGE,
        }
    }
}

/// A keyed lookup miss reads as a missing item; a held key as a conflict.
impl WorthQueryUserFacingRefusal for WorthQueryKeyedEntityRefusal {
    fn user_summary(&self) -> Summary {
        match self.outcome() {
            WorthQueryKeyedEntityOutcome::NotFound => MISSING_ITEM,
            WorthQueryKeyedEntityOutcome::AlreadyExists => ALREADY_EXISTS,
        }
    }
}
