//! Existing callback denial topology retains source and execution causes during original continuation.
use super::WorthQueryRecordedCompletionDenial as Cause;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryInboundAdmissionDenial as Denial;
use crate::domain_computation::primary_graph::provider::{
    WorthQueryCommittedDispatchOutboxReadDenial as Outbox,
    WorthQueryInboundCompletionReadDenial as Read, WorthQueryInboundTerminalIndexDenial as Index,
};

impl Cause {
    pub(in crate::domain_computation::primary_graph) fn into_inbound_denial(&self) -> Denial {
        match self {
            Self::World(cause) => world(*cause),
            Self::Refused(cause) => {
                use worth_runtime_world::facade::WorldRecoveredContinuationDenial as Refusal;
                match cause {
                    Refusal::Admission(cause) => world(*cause),
                    Refusal::PredecessorChanged => Denial::RecoveryStaleProduct,
                    Refusal::Durable(_)
                    | Refusal::Readmission(_)
                    | Refusal::History(_)
                    | Refusal::Retention(_)
                    | Refusal::NoOwnerEffect
                    | Refusal::OwnerEffectsMismatch { .. } => Denial::RecoveryUnavailable,
                }
            }
            Self::Source(cause) => match cause {
                Read::SourceRead(cause) => Denial::OwnerReadDenied(*cause),
                Read::ReadRequest(cause) => read_request(*cause),
                Read::AmbiguousCorrelation
                | Read::Malformed
                | Read::CommitUnavailable
                | Read::ReconstructionWorkExhausted => Denial::RecoveryUnavailable,
            },
            Self::Terminal(cause) => match cause {
                Index::SourceRead(cause) => Denial::OwnerReadDenied(*cause),
                Index::ReadRequest(cause) => read_request(*cause),
                Index::IndexUnavailable
                | Index::ReconstructionWorkExhausted
                | Index::WorldPairMismatch
                | Index::NotExpired => Denial::TerminalCleanupUnavailable,
            },
            Self::Pending | Self::EvidenceRequired | Self::Indeterminate => {
                Denial::RecoveryUnavailable
            }
            Self::PublicationMismatch => Denial::ForeignOwner,
        }
    }
}

fn read_request(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Denial {
    Denial::OwnerReadDenied(Outbox::ReadExecution(
        crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, None)))
}

fn world(cause: worth_runtime_world::facade::RuntimeWorldRecoveryDenial) -> Denial {
    use worth_runtime_world::facade::RuntimeWorldRecoveryDenial as World;
    match cause {
        World::ForeignHandle | World::StaleIncarnation | World::OriginalSourceMismatch => {
            Denial::ForeignOwner
        }
        World::Busy
        | World::CallerCapabilityLive
        | World::ContinuationAlreadyStarted
        | World::ReleaseAlreadyStarted => Denial::PublicationInProgress,
        World::HomeProgramTransitionRequired
        | World::RecoveredCustodyRequired
        | World::ContinuationAlreadyCommitted
        | World::ReopenRequired
        | World::OwnerUnavailable(_)
        | World::MissingRecord
        | World::TooYoung
        | World::ClockRegressed
        | World::SettlementRequired
        | World::SettlementEvidenceUnavailable
        | World::OriginalSourceUnavailable
        | World::OutputCapacityExhausted => Denial::RecoveryUnavailable,
    }
}
