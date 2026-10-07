//! An accepted occurrence reports the cause without changing retry custody.
use super::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundPendingReason as Pending,
};
impl Denial {
    pub(in crate::domain_computation::primary_graph) fn pending_reason(self) -> Pending {
        match self {
            Denial::PublicationExecutionDenied { stage, kind } => {
                Pending::ExecutionDenied { stage, kind }
            }
            Denial::PublicationExecutionControlStopped { stage, kind } => {
                Pending::ExecutionControlStopped { stage, kind }
            }
            Denial::SourceRevoked => Pending::SourceRevoked,
            Denial::TerminalCleanupUnavailable => Pending::TerminalCleanupUnavailable,
            Denial::RecoveryUnavailable => Pending::RecoveryUnavailable,
            Denial::CorrelationAlreadyOwned => Pending::CorrelationConflict,
            Denial::ForeignVerifier
            | Denial::Oversized
            | Denial::Verification(_)
            | Denial::IncompatibleMeaning
            | Denial::Expired
            | Denial::ValidityWindowExceeded
            | Denial::TimeUnavailable
            | Denial::UnknownCorrelation
            | Denial::RetryBeforeAcceptance
            | Denial::ForeignOwner
            | Denial::OriginalDispatchHasNoInboundSupport
            | Denial::UnsupportedOutbox
            | Denial::MessageIdentityConflict
            | Denial::AuthenticatedPermanent(_)
            | Denial::CapacityExhausted
            | Denial::PublicationInProgress
            | Denial::PublicationRetryRequired
            | Denial::RecoveryStaleProduct
            | Denial::SourceRetired
            | Denial::OwnerReadDenied(_) => Pending::OwnerRetryRequired,
        }
    }
}
