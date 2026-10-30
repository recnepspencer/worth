//! One model-owned continuation follows native registration, staging, adoption and delivery.
use super::super::{
    attempts::{data::*, recording},
    authentication::Caller,
};
use super::{
    candidate::{
        self,
        adoption::{Adopted, Performed, PreviousCloseFailure},
        PreparationFailure, ReadyToAdopt, Staging,
    },
    model::Model,
    storage::{self, operations::*, LoadedProposal, Storage},
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution,
};

mod delivery;
mod ingress;
mod registration;
mod staging;

pub(super) struct Attempt<'host> {
    loaded: LoadedProposal,
    key: u64,
    phase: Phase<'host>,
}
enum Phase<'host> {
    Claim(ClaimOperation),
    ClaimCompleted(ClaimCompleted),
    ClaimTerminal(ClaimTerminal),
    Registered(Registration),
    PreparingFailed {
        registration: Registration,
        failure: PreparationFailure<'host>,
    },
    Staging {
        registration: Registration,
        staging: Staging<'host>,
    },
    Ready {
        registration: Registration,
        ready: ReadyToAdopt<'host>,
    },
    Adopted {
        registration: Registration,
        adopted: Adopted<'host>,
        delivery: Delivery,
    },
    PreviousClose {
        failure: PreviousCloseFailure<'host>,
        result: StoredResult,
    },
}
struct Registration {
    claim: ClaimReference,
    evidence: ClaimCompletion,
}
enum Delivery {
    Needed,
    Operation(ResultOperation),
    Completed(ResultCompleted),
    Terminal(ResultTerminal),
    Written(StoredResult),
}
pub struct Accepted {
    pub result: StoredResult,
    pub performed: Performed,
}
pub enum Event {
    Progress,
    Staged(candidate::StageEvent),
    Adopted,
    Completed(Accepted),
    Cached(StoredResult),
    Stopped,
}
impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Progress => f.write_str("Progress"),
            Self::Staged(event) => f.debug_tuple("Staged").field(event).finish(),
            Self::Adopted => f.write_str("Adopted"),
            Self::Completed(_) => f.write_str("Completed"),
            Self::Cached(_) => f.write_str("Cached"),
            Self::Stopped => f.write_str("Stopped"),
        }
    }
}
#[derive(Debug)]
pub enum Failure {
    Admission(storage::AdmissionFailure),
    Resolution(WorthQueryApplicationRecoveryRequestDenial),
    IntentDrift,
    Busy,
    OriginalContinuationRetained,
    ContinuationUnavailable,
    Eligibility(candidate::PreparationRefusal),
    Construction(ConstructionFailure),
    ClaimRead(storage::claims::ClaimReadFailure),
    ResultRead(storage::results::ResultReadFailure),
    MetadataAdvance(WorthQuerySourceOnlyDenial),
    OperationClose(WorthQueryOperationCloseDenial),
    Correspondence(recording::CorrespondenceFailure),
    Adoption {
        cause: candidate::adoption::AdoptionRefusal,
        handoff:
            Option<worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial>,
    },
    Advancement(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial),
}
pub enum View<'a, 'host> {
    Claim(&'a ClaimOperation),
    ClaimCompleted(&'a ClaimCompleted),
    ClaimTerminal(&'a ClaimTerminal),
    Registered,
    Preparation(&'a PreparationFailure<'host>),
    Staging(&'a Staging<'host>),
    Ready,
    Adopted {
        adopted: &'a Adopted<'host>,
        result: Option<&'a StoredResult>,
    },
    ResultTerminal {
        adopted: &'a Adopted<'host>,
        terminal: &'a ResultTerminal,
    },
    PreviousClose(&'a PreviousCloseFailure<'host>),
}
impl std::fmt::Debug for Attempt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingAttempt")
    }
}
impl<'host> Attempt<'host> {
    fn matches(&self, loaded: &LoadedProposal, key: u64) -> bool {
        self.key == key && self.loaded == *loaded
    }
    fn view(&self) -> View<'_, 'host> {
        match &self.phase {
            Phase::Claim(owner) => View::Claim(owner),
            Phase::ClaimCompleted(owner) => View::ClaimCompleted(owner),
            Phase::ClaimTerminal(owner) => View::ClaimTerminal(owner),
            Phase::Registered(_) => View::Registered,
            Phase::PreparingFailed { failure, .. } => View::Preparation(failure),
            Phase::Staging { staging, .. } => View::Staging(staging),
            Phase::Ready { .. } => View::Ready,
            Phase::Adopted {
                adopted, delivery, ..
            } => match delivery {
                Delivery::Terminal(terminal) => View::ResultTerminal { adopted, terminal },
                other => View::Adopted {
                    adopted,
                    result: match other {
                        Delivery::Written(result) => Some(result),
                        _ => None,
                    },
                },
            },
            Phase::PreviousClose { failure, .. } => View::PreviousClose(failure),
        }
    }
}
