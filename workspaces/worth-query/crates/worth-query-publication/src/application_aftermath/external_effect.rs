use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryExternalDispatchPostureKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedExternalEffectPosture {
    NotDeclared,
    PendingDispatch,
    Completed,
    Acknowledged,
    /// The external owner proved the dispatched effect did not happen.
    ProvenNoEffect(WorthQueryPublishedExternalEffectNoEffect),
    Unresolved(WorthQueryPublishedExternalEffectFailure),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedExternalEffectPostureKind {
    NotDeclared,
    PendingDispatch,
    Completed,
    Acknowledged,
    ProvenNoEffect,
    Unresolved,
}

/// Why this attempt is known to have changed no external state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedExternalEffectNoEffect {
    /// The external owner decoded the payload and refused it.
    PayloadRejected,
    /// The attempt did not begin external contact.
    ContactNotStarted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedExternalEffectFailure {
    CompletionExecutionRequest(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Timeout,
    Disconnect,
    LostResponse,
    DuplicatedAcknowledgement,
    InitialDispatchOwnerReadDenied(
        worth_query_execution::facade::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial,
    ),
    InitialDispatchAttemptAdmissionDenied(
        worth_query_execution::facade::primary_graph::WorthQueryExternalDispatchAttemptDenial,
    ),
    InitialDispatchAlreadyCompleted,
    CompletionPublicationPending,
    /// Completion remains unresolved with the execution refusal intact.
    CompletionExecutionDenied {
        stage: worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: worth_query_execution::facade::installed::provider_session::WorthQueryProviderSessionDenialKind,
    },
    /// Completion remains unresolved after this execution control stop.
    CompletionExecutionControlStopped {
        stage: worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: worth_query_execution::facade::installed::provider_session::WorthQueryProviderSessionControlStopKind,
    },
    CompletionAllocationDenied {
        stage: worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: worth_query_execution::facade::runtime::ExecutionAllocationDenialClass,
        requested_payload_bytes: Option<u64>,
    },
    CompletionStagingCardinalityOverflow,
    CompletionInputDirectoryAllocationDenied { requested_batches: usize },
    InitialDispatchTerminalIndexUnavailable,
    InitialDispatchCanonicalDerivationDenied,
    InitialDispatchTimeObservationDenied,
    UnsupportedProtocolVersion {
        produced: u32,
        posture: WorthQueryPublishedUnsupportedProtocolVersionPosture,
    },
    UnknownProviderOutcome,
    ObservationDerivationDenied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedUnsupportedProtocolVersionPosture {
    PredatesWindow,
    ExceedsWindow,
    Retired,
}

impl WorthQueryPublishedExternalEffectPosture {
    pub const fn kind(self) -> WorthQueryPublishedExternalEffectPostureKind {
        match self {
            Self::NotDeclared => WorthQueryPublishedExternalEffectPostureKind::NotDeclared,
            Self::PendingDispatch => WorthQueryPublishedExternalEffectPostureKind::PendingDispatch,
            Self::Completed => WorthQueryPublishedExternalEffectPostureKind::Completed,
            Self::Acknowledged => WorthQueryPublishedExternalEffectPostureKind::Acknowledged,
            Self::ProvenNoEffect(_) => WorthQueryPublishedExternalEffectPostureKind::ProvenNoEffect,
            Self::Unresolved(_) => WorthQueryPublishedExternalEffectPostureKind::Unresolved,
        }
    }

    pub const fn failure(self) -> Option<WorthQueryPublishedExternalEffectFailure> {
        match self {
            Self::Unresolved(failure) => Some(failure),
            Self::NotDeclared
            | Self::PendingDispatch
            | Self::Completed
            | Self::Acknowledged
            | Self::ProvenNoEffect(_) => None,
        }
    }

    pub const fn proven_no_effect(self) -> Option<WorthQueryPublishedExternalEffectNoEffect> {
        match self {
            Self::ProvenNoEffect(cause) => Some(cause),
            Self::NotDeclared
            | Self::PendingDispatch
            | Self::Completed
            | Self::Acknowledged
            | Self::Unresolved(_) => None,
        }
    }
}

pub(super) fn publish_external_effect(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryPublishedExternalEffectPosture {
    match receipt.external_dispatch() {
        Some(dispatch) => match dispatch.posture().kind() {
            WorthQueryExternalDispatchPostureKind::Completed => {
                WorthQueryPublishedExternalEffectPosture::Completed
            }
            WorthQueryExternalDispatchPostureKind::Acknowledged => {
                WorthQueryPublishedExternalEffectPosture::Acknowledged
            }
            WorthQueryExternalDispatchPostureKind::ProvenNoEffect => {
                use worth_query_execution::facade::primary_graph::ExternalEffectNoEffectCause as Cause;
                let proof = dispatch
                    .posture()
                    .proven_no_effect()
                    .expect("a proven-no-effect posture retains its exact proof");
                WorthQueryPublishedExternalEffectPosture::ProvenNoEffect(match proof.cause() {
                    Cause::PayloadRejected => {
                        WorthQueryPublishedExternalEffectNoEffect::PayloadRejected
                    }
                    Cause::ContactNotStarted => {
                        WorthQueryPublishedExternalEffectNoEffect::ContactNotStarted
                    }
                })
            }
            WorthQueryExternalDispatchPostureKind::Unresolved => {
                match dispatch.posture().classification() {
                    Some(classification) => WorthQueryPublishedExternalEffectPosture::Unresolved(
                        publish_failure(classification.fault()),
                    ),
                    None => WorthQueryPublishedExternalEffectPosture::Unresolved(
                        WorthQueryPublishedExternalEffectFailure::UnknownProviderOutcome,
                    ),
                }
            }
        },
        None => match receipt.external_dispatch_preparation_denial() {
            Some(denial) => WorthQueryPublishedExternalEffectPosture::Unresolved(
                publish_preparation_failure(denial),
            ),
            None if receipt.dispatch_outbox().is_some() => {
                WorthQueryPublishedExternalEffectPosture::PendingDispatch
            }
            None => WorthQueryPublishedExternalEffectPosture::NotDeclared,
        },
    }
}

const fn publish_preparation_failure(
    denial: worth_query_execution::facade::primary_graph::WorthQueryExternalDispatchPreparationDenial,
) -> WorthQueryPublishedExternalEffectFailure {
    use worth_query_execution::facade::primary_graph::WorthQueryExternalDispatchPreparationDenial as Execution;

    match denial {
        Execution::CompletionExecutionRequest(cause) => {
            WorthQueryPublishedExternalEffectFailure::CompletionExecutionRequest(cause)
        }
        Execution::OwnerReadDenied(read) => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchOwnerReadDenied(read)
        }
        Execution::AttemptAdmissionDenied(attempt) => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchAttemptAdmissionDenied(attempt)
        }
        Execution::AlreadyCompleted => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchAlreadyCompleted
        }
        Execution::CompletionPublicationPending => {
            WorthQueryPublishedExternalEffectFailure::CompletionPublicationPending
        }
        Execution::CompletionExecutionDenied { stage, kind } => {
            WorthQueryPublishedExternalEffectFailure::CompletionExecutionDenied { stage, kind }
        }
        Execution::CompletionExecutionControlStopped { stage, kind } => {
            WorthQueryPublishedExternalEffectFailure::CompletionExecutionControlStopped {
                stage,
                kind,
            }
        }
        Execution::CompletionAllocationDenied {
            stage,
            kind,
            requested_payload_bytes,
        } => WorthQueryPublishedExternalEffectFailure::CompletionAllocationDenied {
            stage,
            kind,
            requested_payload_bytes,
        },
        Execution::CompletionStagingCardinalityOverflow => {
            WorthQueryPublishedExternalEffectFailure::CompletionStagingCardinalityOverflow
        }
        Execution::CompletionInputDirectoryAllocationDenied { requested_batches } => {
            WorthQueryPublishedExternalEffectFailure::CompletionInputDirectoryAllocationDenied {
                requested_batches,
            }
        }
        Execution::TerminalIndexUnavailable => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchTerminalIndexUnavailable
        }
        Execution::CanonicalDerivationDenied => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchCanonicalDerivationDenied
        }
        Execution::TimeObservationDenied => {
            WorthQueryPublishedExternalEffectFailure::InitialDispatchTimeObservationDenied
        }
    }
}

const fn publish_failure(
    failure: worth_query_execution::facade::primary_graph::ExternalRailTransportFault,
) -> WorthQueryPublishedExternalEffectFailure {
    use worth_query_execution::facade::primary_graph::ExternalRailTransportFault as Execution;

    match failure {
        Execution::Timeout => WorthQueryPublishedExternalEffectFailure::Timeout,
        Execution::Disconnect => WorthQueryPublishedExternalEffectFailure::Disconnect,
        Execution::LostResponse => WorthQueryPublishedExternalEffectFailure::LostResponse,
        Execution::DuplicatedAcknowledgement => {
            WorthQueryPublishedExternalEffectFailure::DuplicatedAcknowledgement
        }
        Execution::UnsupportedProtocolVersion(unsupported) => {
            use worth_foundational::facade::BoundaryProtocolUnsupportedVersionPosture as Posture;
            let posture = match unsupported.posture() {
                Posture::PredatesWindow => {
                    WorthQueryPublishedUnsupportedProtocolVersionPosture::PredatesWindow
                }
                Posture::ExceedsWindow => {
                    WorthQueryPublishedUnsupportedProtocolVersionPosture::ExceedsWindow
                }
                Posture::Retired => WorthQueryPublishedUnsupportedProtocolVersionPosture::Retired,
            };
            WorthQueryPublishedExternalEffectFailure::UnsupportedProtocolVersion {
                produced: unsupported.produced().get(),
                posture,
            }
        }
        Execution::UnknownProviderOutcome => {
            WorthQueryPublishedExternalEffectFailure::UnknownProviderOutcome
        }
        Execution::ObservationDerivationDenied => {
            WorthQueryPublishedExternalEffectFailure::ObservationDerivationDenied
        }
    }
}

#[cfg(test)]
mod tests;
