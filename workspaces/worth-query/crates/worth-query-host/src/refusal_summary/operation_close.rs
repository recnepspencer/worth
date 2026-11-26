//! Refusals to close an operation, such as discarding a held source command.
//!
//! Work still settling (native obligations, busy cleanup, pending recovery)
//! is `Unavailable` and retryable; capacity stops are `Capacity`; storage
//! faults are `Storage`; foreign, stale or inconsistent owners are `Internal`.
use worth_query_execution::facade::domain_computation::WorthQueryManagedRunCleanupFailureKind as Native;
use worth_query_execution::facade::primary_graph::{
    WorthQueryMutationCleanupFailureKind as Cleanup, WorthQueryOriginalOperationDenial as Original,
    WorthQueryOutputReadCloseDenial as OutputRead,
};
use worth_query_execution::facade::product::{
    WorthQueryProductBranchOwnerCleanupDenial as Owner,
    WorthQueryProductUnpublishedRecoveryReleaseDenial as Recovery,
};
use worth_query_execution::facade::query_obligations::QueryObligationDenial as Obligation;
use worth_query_publication::facade::application_entry::{
    WorthQueryOperationCloseDenial, WorthQueryRequiredOutputPreparationDenial as Outputs,
};

use super::request::{BUSY, CAPACITY, INTERNAL, STORAGE, UNSUPPORTED, VERSION_MISMATCH};
use super::stops::{principal_resolution, work};
use super::summary::{WorthQueryRefusalSummary as Summary, WorthQueryUserFacingRefusal};

impl WorthQueryUserFacingRefusal for WorthQueryOperationCloseDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Original(cause) => original(cause),
            Self::ManagedCleanup(kind) => cleanup(kind),
            Self::CleanupCapacity => CAPACITY,
            Self::NativeObligationRetained => BUSY,
            Self::OutputRead(cause) => output_read(cause),
            Self::Outputs(cause) => outputs(cause),
            Self::Execution(cause) => cause.user_summary(),
            Self::Recovery(cause) => recovery(cause),
            Self::Owner(cause) => owner(cause),
            Self::DeferredFixedCompletionUnsupported { .. } => UNSUPPORTED,
            Self::FreshRequestMismatch | Self::HistoryGroup(_) => INTERNAL,
        }
    }
}

fn original(cause: &Original) -> Summary {
    match cause {
        Original::Principal(denial) => principal_resolution(denial.kind()),
        Original::Scope(denial) => denial.user_summary(),
        Original::QueryAdmission(denial) => denial.user_summary(),
        Original::QueryExecution(denial) => denial.user_summary(),
        Original::Execution(denial) => denial.user_summary(),
        Original::Obligation(denial) => obligation(denial),
        Original::OwnershipMaterialUnavailable
        | Original::StaleIncarnation
        | Original::ForeignHome => INTERNAL,
    }
}

fn obligation(cause: &Obligation) -> Summary {
    match cause {
        Obligation::UnsupportedVersion(_) => VERSION_MISMATCH,
        Obligation::MalformedFrame
        | Obligation::Checksum
        | Obligation::PrefixMismatch
        | Obligation::Storage(_) => STORAGE,
        Obligation::IdentityExhausted => CAPACITY,
        Obligation::Execution(cause) => work(cause),
        Obligation::AdoptionReadmissionRequired
        | Obligation::ConditionalCleanupPending { .. }
        | Obligation::ManagedRunPending { .. }
        | Obligation::OwnedAsyncPending { .. }
        | Obligation::OwnedAsyncQueuePending { .. }
        | Obligation::OwnerUnavailable
        | Obligation::IncarnationNotEnded
        | Obligation::PublicationPending
        | Obligation::PublicationNoEffect
        | Obligation::PublicationUnresolved => BUSY,
        Obligation::ForeignHome
        | Obligation::DuplicateProgram
        | Obligation::UnrosteredProgram(_)
        | Obligation::StatusMismatch
        | Obligation::IllegalStatusTransition
        | Obligation::ObligationIdentityMismatch
        | Obligation::ObligationSubjectMismatch
        | Obligation::SupportUnavailable
        | Obligation::UnknownObligation
        | Obligation::DuplicateObligation
        | Obligation::CleanupReleaseUnrecorded { .. }
        | Obligation::CleanupInconsistent { .. }
        | Obligation::MissingOriginalInterpretationSource { .. }
        | Obligation::OriginalInterpretationProgramMismatch { .. }
        | Obligation::OwnedAsyncMandatory { .. }
        | Obligation::StaleIncarnation
        | Obligation::PublicationGenerationMismatch => INTERNAL,
    }
}

fn cleanup(kind: &Cleanup) -> Summary {
    match kind {
        Cleanup::Native(Native::ProviderMemoryRetained | Native::QueueRelease(_)) => BUSY,
        Cleanup::Native(Native::BridgeFinalization(_))
        | Cleanup::AffinityInconsistent
        | Cleanup::CompletionInconsistent => INTERNAL,
    }
}

fn output_read(cause: &OutputRead) -> Summary {
    match cause {
        OutputRead::Execution(denial) => denial.user_summary(),
        OutputRead::Recovery(denial) => recovery(denial),
        OutputRead::Owner(denial) => owner(denial),
        OutputRead::NativePublicationRetained | OutputRead::ProducerRecoveryRetained => BUSY,
        OutputRead::ForeignDemand | OutputRead::Cleanup(_) => INTERNAL,
    }
}

fn outputs(cause: &Outputs) -> Summary {
    match cause {
        Outputs::SourceQuery(denial) => denial.user_summary(),
        Outputs::Dependencies(_) | Outputs::ReadObservation(_) => BUSY,
        Outputs::ForeignProgram
        | Outputs::UndeclaredOutputRoot
        | Outputs::MissingPerformedDelivery
        | Outputs::MissingSource
        | Outputs::DemandExecution(_)
        | Outputs::Demand(_)
        | Outputs::Connection(_)
        | Outputs::Closed => INTERNAL,
    }
}

fn recovery(cause: &Recovery) -> Summary {
    match cause {
        Recovery::Execution(denial) => denial.user_summary(),
        Recovery::CleanupCapacityExhausted => CAPACITY,
        Recovery::RecoveredPending
        | Recovery::RecoveredIndeterminate
        | Recovery::ProviderBusy
        | Recovery::World(_) => BUSY,
        Recovery::OriginalBinding(_)
        | Recovery::OriginalBindingMismatch
        | Recovery::RecoveredRefused
        | Recovery::RecoveredEvidenceRequired => INTERNAL,
    }
}

fn owner(cause: &Owner) -> Summary {
    match cause {
        Owner::Execution(denial) => denial.user_summary(),
        Owner::CleanupBusy
        | Owner::WorldHistoryStillRetained
        | Owner::RelationalOperationsStillActive
        | Owner::RelationalDurabilityUnresolved => BUSY,
        Owner::CleanupUnavailable
        | Owner::WorldHistoryUnavailable
        | Owner::RelationalOwnerDenied
        | Owner::SignalOwnerDenied
        | Owner::DurableSignalRetirementRequired => INTERNAL,
    }
}
