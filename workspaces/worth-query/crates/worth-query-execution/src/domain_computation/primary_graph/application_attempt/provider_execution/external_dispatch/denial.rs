//! Exact refusals before a post-commit transport call could be made.
//!
//! Every runtime refusal of a physical attempt keeps its own kind, from the
//! runtime's admission through the outstanding-dispatch lease, so a caller
//! never guesses the cause behind one shared kind.

use crate::domain_computation::application_aftermath::{
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryExternalDispatchAdmissionDenial;
use crate::domain_computation::primary_graph::provider::WorthQueryOutstandingInFlightDenial;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial;

use crate::domain_computation::primary_graph::application_runtime::InstalledTransportPendingReason;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage as Stage;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as SessionKind,
};

/// Why a host could not install an external-effect transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalTransportInstallationDenial {
    /// A transport is already installed; it is never replaced in flight.
    AlreadyInstalled,
}

/// Why this runtime could not admit one runtime-affine physical attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchAttemptDenial {
    /// The committed outbox was observed on another Relational runtime.
    ForeignRelationalRuntime,
    /// The committed publication belongs to another product world.
    ForeignProductWorld,
    /// The outbox commit and its product publication name different commits.
    PublicationCommitMismatch,
    /// An inbound-completing effect carries no co-committed operation slot,
    /// so its completion could never be found by owner maintenance.
    InboundOperationSlotMissing,
    /// This runtime has no physical attempt identities left.
    AttemptIdentityExhausted,
    /// The outstanding-dispatch owner holds no entry for this effect.
    OutstandingDispatchMissing,
    /// The original dispatch's publication has not settled yet.
    OriginalPublicationPending,
    /// The outstanding-dispatch entry disagrees with the committed original.
    OutstandingDispatchMismatch,
    /// The installed limit on concurrent physical sends for this effect is
    /// reached; it frees when an in-flight send settles.
    InFlightCapacityExhausted,
}

/// Exact failure before an initial post-commit transport call could be made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchPreparationDenial {
    OwnerReadDenied(WorthQueryCommittedDispatchOutboxReadDenial),
    AttemptAdmissionDenied(WorthQueryExternalDispatchAttemptDenial),
    AlreadyCompleted,
    CompletionPublicationPending,
    /// Completion publication is retryable; its execution refusal stays typed.
    CompletionExecutionDenied {
        stage: Stage,
        kind: SessionKind,
    },
    /// Completion publication is retryable after this control stop.
    CompletionExecutionControlStopped {
        stage: Stage,
        kind: Control,
    },
    TerminalIndexUnavailable,
    CanonicalDerivationDenied,
    TimeObservationDenied,
}

/// Why an admitted re-dispatch could not run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalRedispatchDenial {
    /// Fresh effect authority for the handle failed before transport: the
    /// handle already ended, it expired, or the authority is not its own.
    FreshAuthority(WorthQueryRecoveryHandleDenialKind),
    /// The admitted request was cancelled before transport.
    AdmissionCancelled,
    /// The admitted request reached its deadline before transport.
    AdmissionDeadlineExceeded,
    /// The admitted principal's authentication expired before transport.
    AdmissionAuthenticationExpired,
    /// The admission was minted by another runtime or installed binding.
    ForeignAdmission,
    RecoveryNotAdmitted,
    /// The live handle binding carries no co-committed outbox record.
    BindingOutboxMissing,
    /// No host transport is installed on this runtime.
    TransportNotInstalled,
    /// Relational could not establish the exact committed owner row.
    OwnerReadDenied(WorthQueryCommittedDispatchOutboxReadDenial),
    /// This runtime could not mint a runtime-affine physical attempt.
    AttemptAdmissionDenied(WorthQueryExternalDispatchAttemptDenial),
    AlreadyCompleted,
    CompletionPublicationPending,
    /// Completion publication is retryable; its execution refusal stays typed.
    CompletionExecutionDenied {
        stage: Stage,
        kind: SessionKind,
    },
    /// Completion publication is retryable after this control stop.
    CompletionExecutionControlStopped {
        stage: Stage,
        kind: Control,
    },
    TerminalIndexUnavailable,
    /// Canonical derivation for the dispatch event identity failed.
    CanonicalDerivationDenied,
    /// The installed runtime clock could not classify this physical attempt.
    TimeObservationDenied,
}

/// Each re-dispatch refusal keeps its own recovery denial kind, so a caller
/// never has to guess the cause behind one shared kind.
impl From<WorthQueryExternalRedispatchDenial> for WorthQueryRecoveryHandleDenial {
    fn from(denial: WorthQueryExternalRedispatchDenial) -> Self {
        use WorthQueryExternalRedispatchDenial as Redispatch;
        use WorthQueryRecoveryHandleDenialKind as Kind;
        WorthQueryRecoveryHandleDenial::new(match denial {
            Redispatch::FreshAuthority(kind) => kind,
            Redispatch::AdmissionCancelled => Kind::AdmissionCancelled,
            Redispatch::AdmissionDeadlineExceeded => Kind::AdmissionDeadlineExceeded,
            Redispatch::AdmissionAuthenticationExpired => Kind::AdmissionAuthenticationExpired,
            Redispatch::ForeignAdmission => Kind::ForeignRuntime,
            Redispatch::RecoveryNotAdmitted => Kind::RecoveryNotAdmitted,
            Redispatch::BindingOutboxMissing => Kind::DispatchOutboxMissing,
            Redispatch::TransportNotInstalled => Kind::TransportNotInstalled,
            Redispatch::OwnerReadDenied(read) => Kind::DispatchOwnerReadDenied(read),
            Redispatch::AttemptAdmissionDenied(attempt) => Kind::AttemptAdmissionDenied(attempt),
            Redispatch::AlreadyCompleted => Kind::AlreadyCompleted,
            Redispatch::CompletionPublicationPending => Kind::CompletionPublicationPending,
            Redispatch::CompletionExecutionDenied { stage, kind } => {
                Kind::CompletionExecutionDenied { stage, kind }
            }
            Redispatch::CompletionExecutionControlStopped { stage, kind } => {
                Kind::CompletionExecutionControlStopped { stage, kind }
            }
            Redispatch::TerminalIndexUnavailable => Kind::TerminalIndexUnavailable,
            Redispatch::CanonicalDerivationDenied => Kind::CanonicalDerivationDenied,
            Redispatch::TimeObservationDenied => Kind::TimeObservationDenied,
        })
    }
}

/// The runtime's own admission refusal, each cause kept exactly.
pub(super) const fn from_attempt_admission(
    denial: WorthQueryExternalDispatchAdmissionDenial,
) -> WorthQueryExternalDispatchPreparationDenial {
    use WorthQueryExternalDispatchAdmissionDenial as Admission;
    use WorthQueryExternalDispatchAttemptDenial as Attempt;
    use WorthQueryExternalDispatchPreparationDenial as Preparation;
    match denial {
        Admission::AlreadyCompleted => Preparation::AlreadyCompleted,
        Admission::CompletedTransportRetained => Preparation::CompletionPublicationPending,
        Admission::TerminalIndexUnavailable => Preparation::TerminalIndexUnavailable,
        Admission::ForeignRelationalRuntime => {
            Preparation::AttemptAdmissionDenied(Attempt::ForeignRelationalRuntime)
        }
        Admission::ForeignProductWorld => {
            Preparation::AttemptAdmissionDenied(Attempt::ForeignProductWorld)
        }
        Admission::PublicationCommitMismatch => {
            Preparation::AttemptAdmissionDenied(Attempt::PublicationCommitMismatch)
        }
        Admission::MissingInboundOperationSlot => {
            Preparation::AttemptAdmissionDenied(Attempt::InboundOperationSlotMissing)
        }
        Admission::AttemptIdentityExhausted => {
            Preparation::AttemptAdmissionDenied(Attempt::AttemptIdentityExhausted)
        }
    }
}

/// The outstanding-dispatch lease refusal, each cause kept exactly.
pub(super) const fn from_in_flight(
    denial: WorthQueryOutstandingInFlightDenial,
) -> WorthQueryExternalDispatchPreparationDenial {
    use WorthQueryExternalDispatchAttemptDenial as Attempt;
    use WorthQueryExternalDispatchPreparationDenial as Preparation;
    use WorthQueryOutstandingInFlightDenial as InFlight;
    match denial {
        InFlight::TerminalReached => Preparation::AlreadyCompleted,
        InFlight::Missing => {
            Preparation::AttemptAdmissionDenied(Attempt::OutstandingDispatchMissing)
        }
        InFlight::PendingPublication => {
            Preparation::AttemptAdmissionDenied(Attempt::OriginalPublicationPending)
        }
        InFlight::OriginalMismatch => {
            Preparation::AttemptAdmissionDenied(Attempt::OutstandingDispatchMismatch)
        }
        InFlight::CapacityExhausted => {
            Preparation::AttemptAdmissionDenied(Attempt::InFlightCapacityExhausted)
        }
    }
}

/// Maps the shared preparation step's refusal onto re-dispatch. The owner row
/// is read before that step, so its refusal never reaches this mapping.
pub(super) fn redispatch_preparation(
    denial: WorthQueryExternalDispatchPreparationDenial,
) -> WorthQueryExternalRedispatchDenial {
    use WorthQueryExternalDispatchPreparationDenial as Preparation;
    use WorthQueryExternalRedispatchDenial as Redispatch;
    match denial {
        Preparation::AttemptAdmissionDenied(attempt) => Redispatch::AttemptAdmissionDenied(attempt),
        Preparation::AlreadyCompleted => Redispatch::AlreadyCompleted,
        Preparation::CompletionPublicationPending => Redispatch::CompletionPublicationPending,
        Preparation::CompletionExecutionDenied { stage, kind } => {
            Redispatch::CompletionExecutionDenied { stage, kind }
        }
        Preparation::CompletionExecutionControlStopped { stage, kind } => {
            Redispatch::CompletionExecutionControlStopped { stage, kind }
        }
        Preparation::TerminalIndexUnavailable => Redispatch::TerminalIndexUnavailable,
        Preparation::CanonicalDerivationDenied => Redispatch::CanonicalDerivationDenied,
        Preparation::TimeObservationDenied => Redispatch::TimeObservationDenied,
        Preparation::OwnerReadDenied(read) => Redispatch::OwnerReadDenied(read),
    }
}

#[cfg(test)]
#[path = "denial_tests.rs"]
mod tests;

impl InstalledTransportPendingReason {
    pub(in crate::domain_computation::primary_graph) const fn dispatch_preparation_denial(
        self,
    ) -> WorthQueryExternalDispatchPreparationDenial {
        use InstalledTransportPendingReason as Pending;
        use WorthQueryExternalDispatchPreparationDenial as Preparation;
        match self {
            Pending::ExecutionDenied { stage, kind } => {
                Preparation::CompletionExecutionDenied { stage, kind }
            }
            Pending::ExecutionControlStopped { stage, kind } => {
                Preparation::CompletionExecutionControlStopped { stage, kind }
            }
            Pending::UnknownCompletion
            | Pending::ConcurrentContinuation
            | Pending::PublicationRetryRequired
            | Pending::PublicationAtCapacity
            | Pending::ProductRecoveryRequired
            | Pending::RecoveryStaleProduct
            | Pending::TerminalProtectionUnavailable
            | Pending::TerminalReleaseUnavailable => Preparation::CompletionPublicationPending,
        }
    }
}

impl WorthQueryExternalDispatchPreparationDenial {
    pub(in crate::domain_computation::primary_graph) fn redispatch_denial(
        self,
    ) -> WorthQueryExternalRedispatchDenial {
        redispatch_preparation(self)
    }
}
