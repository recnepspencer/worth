//! A refused completion stays ready for the next bounded maintenance pass.
use super::super::installed_transport::InstalledTransportPublicationDenial as Denial;
use super::InstalledTransportPendingReason as Pending;
use crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionPreparationDenial as Preparation;

impl Denial {
    pub(in crate::domain_computation::primary_graph) fn pending_reason(self) -> Pending {
        match self {
            Denial::CompletionPreparation(preparation) => match preparation {
                Preparation::ExecutionDenied { stage, kind } => {
                    Pending::ExecutionDenied { stage, kind }
                }
                Preparation::ExecutionControlStopped { stage, kind } => {
                    Pending::ExecutionControlStopped { stage, kind }
                }
                Preparation::OriginalOutboxNotAnEntity
                | Preparation::ForeignOrStaleBasis
                | Preparation::StagingUnavailable
                | Preparation::ValidationUnavailable
                | Preparation::PreparationUnavailable
                | Preparation::SnapshotUnavailable
                | Preparation::IndexPreparationUnavailable
                | Preparation::InstalledBindingMismatch
                | Preparation::DispatchOwnerMismatch => Pending::PublicationRetryRequired,
            },
            Denial::PublicationPermit => Pending::PublicationAtCapacity,
            Denial::ForeignRelationalRuntime
            | Denial::ForeignProductWorld
            | Denial::OriginalPublicationCommitMismatch
            | Denial::BranchCoordinationCapacityExhausted
            | Denial::BindingUnavailable
            | Denial::TerminalIndexUnavailable
            | Denial::CorrelationAlreadyOwned
            | Denial::ProductAdmission => Pending::PublicationRetryRequired,
        }
    }
}
