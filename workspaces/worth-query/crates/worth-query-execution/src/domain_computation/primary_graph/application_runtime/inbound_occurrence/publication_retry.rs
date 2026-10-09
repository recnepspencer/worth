//! A publication refusal retains accepted custody for retry.
use super::receive::WorthQueryInboundAdmissionDenial as Retry;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryInboundPublicationDenial as Denial;
use crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionPreparationDenial as Preparation;

impl Denial {
    pub(in crate::domain_computation::primary_graph) fn retry_denial(self) -> Retry {
        match self {
            Denial::CompletionPreparation(preparation) => match preparation {
                Preparation::ExecutionDenied { stage, kind } => {
                    Retry::PublicationExecutionDenied { stage, kind }
                }
                Preparation::ExecutionControlStopped { stage, kind } => {
                    Retry::PublicationExecutionControlStopped { stage, kind }
                }
                Preparation::AllocationDenied { stage, kind, requested_payload_bytes } => Retry::PublicationAllocationDenied { stage, kind, requested_payload_bytes },
                Preparation::StagingAllocationDenied { kind, requested_payload_bytes } => Retry::PublicationAllocationDenied {
                    stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::EffectLowering,
                    kind, requested_payload_bytes,
                },
                Preparation::StagingCardinalityOverflow => Retry::PublicationStagingCardinalityOverflow,
                Preparation::StagingInputDirectoryAllocationDenied { requested_batches } => Retry::PublicationInputDirectoryAllocationDenied { requested_batches },
                Preparation::OriginalOutboxNotAnEntity
                | Preparation::ForeignOrStaleBasis
                | Preparation::StagingUnavailable
                | Preparation::ValidationUnavailable
                | Preparation::PreparationUnavailable
                | Preparation::SnapshotUnavailable
                | Preparation::IndexPreparationUnavailable
                | Preparation::InstalledBindingMismatch
                | Preparation::DispatchOwnerMismatch => Retry::PublicationRetryRequired,
            },
            Denial::CorrelationAlreadyOwned => Retry::CorrelationAlreadyOwned,
            Denial::ForeignRelationalRuntime
            | Denial::ForeignProductWorld
            | Denial::OriginalPublicationCommitMismatch
            | Denial::BranchCoordinationCapacityExhausted
            | Denial::ProductAdmission
            | Denial::TerminalIndexUnavailable => Retry::PublicationRetryRequired,
        }
    }
}
