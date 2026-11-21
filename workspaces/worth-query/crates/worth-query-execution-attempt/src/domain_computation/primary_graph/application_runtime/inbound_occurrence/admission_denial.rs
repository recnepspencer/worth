//! Inbound completion admission denial vocabulary.
use super::receive_denial::WorthQueryInboundAuthenticatedPermanentDenial;
use crate::domain_computation::application_aftermath::external_effect::inbound::WorthQueryInboundVerificationDenial;
use crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial;

/// Why an authenticated completion could not enter or finish owner custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundAdmissionDenial {
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    ForeignVerifier,
    Oversized,
    Verification(WorthQueryInboundVerificationDenial),
    IncompatibleMeaning,
    Expired,
    ValidityWindowExceeded,
    TimeUnavailable,
    UnknownCorrelation,
    RetryBeforeAcceptance,
    ForeignOwner,
    OriginalDispatchHasNoInboundSupport,
    UnsupportedOutbox,
    MessageIdentityConflict,
    CorrelationAlreadyOwned,
    AuthenticatedPermanent(WorthQueryInboundAuthenticatedPermanentDenial),
    CapacityExhausted,
    TerminalCleanupUnavailable,
    PublicationInProgress,
    PublicationRetryRequired,
    PublicationAllocationDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: worth_execution::ExecutionAllocationDenialClass,
        requested_payload_bytes: Option<u64>,
    },
    PublicationStagingCardinalityOverflow,
    PublicationInputDirectoryAllocationDenied {
        requested_batches: usize,
    },
    /// Publication remains retryable; the execution owner retains its cause.
    PublicationExecutionDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: crate::domain_computation::WorthQueryProviderSessionDenialKind,
    },
    /// Cancellation or deadline during preparation remains retryable.
    PublicationExecutionControlStopped {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    },
    RecoveryStaleProduct,
    RecoveryUnavailable,
    SourceRetired,
    SourceRevoked,
    OwnerReadDenied(WorthQueryCommittedDispatchOutboxReadDenial),
}

impl WorthQueryInboundAdmissionDenial {
    pub(crate) fn advancement(
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    ) -> Self {
        let stage = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::ResourceAdmission;
        match cause.provider_denial_cause() {
            Ok(kind) => Self::PublicationExecutionDenied { stage, kind },
            Err(kind) => Self::PublicationExecutionControlStopped { stage, kind },
        }
    }
}
