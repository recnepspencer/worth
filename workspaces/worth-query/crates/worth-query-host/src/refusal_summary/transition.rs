//! Directory Home program transition (upgrade) refusals.
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;

use crate::application_home::{
    WorthQueryHomeProgramTransitionDenial, WorthQueryHomeProgramTransitionOpenPhase as Phase,
    WorthQueryHomeProgramTransitionOpenRefused,
};
use crate::program_transition::WorthQueryProgramTransitionOpenDenial;

use super::request::{CAPACITY, INTERNAL, VERSION_MISMATCH};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const UPGRADE_UNFINISHED: Summary = Summary::with_message(
    Category::Storage,
    "The app couldn't finish upgrading its saved data. Try opening it again.",
);
const DIFFERENT_VERSION: Summary = Summary::with_message(
    Category::Unsupported,
    "This data was saved by a different version of the app, so it can't be opened here.",
);

impl WorthQueryUserFacingRefusal for WorthQueryHomeProgramTransitionDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::DirectoryRequired => Summary::with_message(
                Category::Unsupported,
                "Only data saved on this device can be upgraded.",
            ),
            Self::BindingAdmission(_) | Self::Binding(_) => VERSION_MISMATCH,
            Self::FiniteResourcesRequired
            | Self::InvalidPlan
            | Self::Author(_)
            | Self::AuthoredBody(_)
            | Self::PlanningRequest(_) => INTERNAL,
            Self::StoppedOpen { .. }
            | Self::StoppedPublication { .. }
            | Self::StoppedContinuation { .. }
            | Self::Request(_) => CAPACITY,
            Self::Source(_)
            | Self::Workflow(_)
            | Self::Native(_)
            | Self::Body(_)
            | Self::Staging(_)
            | Self::Commit(_)
            | Self::Indexes(_)
            | Self::Intent(_)
            | Self::Plan(_)
            | Self::IntentOutcome(_)
            | Self::Open(_)
            | Self::OpenPreparation { .. }
            | Self::NativePreparation(_)
            | Self::Publication(_)
            | Self::Completion(_)
            | Self::Resumption(_)
            | Self::UnopenedStep(_)
            | Self::ResumedOpen(_)
            | Self::Continuation(_)
            | Self::Observation(_)
            | Self::Composition(_)
            | Self::BridgeProgramme(_) => UPGRADE_UNFINISHED,
        }
    }
}

impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>
    WorthQueryUserFacingRefusal for WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>
{
    fn user_summary(&self) -> Summary {
        match self.phase() {
            Phase::Predecessor { refused, .. } => refused.user_summary(),
            Phase::Successor(refused) => refused.user_summary(),
            Phase::Configuration(cause)
            | Phase::BeforeCompilation { cause, .. }
            | Phase::Transition { cause, .. } => cause.user_summary(),
            Phase::Compilation { .. } => INTERNAL,
            Phase::Promotion { .. } => UPGRADE_UNFINISHED,
        }
    }
}

impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>
    WorthQueryUserFacingRefusal for WorthQueryProgramTransitionOpenDenial<A, AP, B, BP>
{
    fn user_summary(&self) -> Summary {
        match self {
            Self::Declaration(_) | Self::InvalidConfiguration(_) => INTERNAL,
            Self::PredecessorMismatch { .. } => DIFFERENT_VERSION,
            Self::EntityScanLimitExceeded { .. } | Self::BudgetOrCancellationStopped { .. } => {
                CAPACITY
            }
            Self::Host(refused) => refused.user_summary(),
        }
    }
}
