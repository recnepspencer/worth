//! Workflow audit pages: a page's own stop, classified through each native
//! lookup, index and execution cause, so a work budget, deadline or storage
//! fault never renders as an internal defect.
use worth_query_execution::facade::{
    application_discovery::WorthQueryWorkflowReadDenial as WorkflowRead,
    primary_graph::{
        WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind as Attempt,
    },
};
use worth_query_publication::facade::application_entry::WorthQueryWorkflowObservationDenial;
use worth_relational::facade::{
    indexes::{
        BoundedEntityFieldLookupDenialKind as FieldLookup,
        BoundedRelatedEntityOrderedLookupDenialKind as OrderedLookup,
        BoundedRelatedEntityPrefixDenial as Page, DerivedIndexMaintenanceDenialKind as Maintenance,
    },
    mvcc::{CompanionPreflightStop as Preflight, RelationalOperationInterruption as Interruption},
    runtime::RelationalPinnedReadAdmissionDenial as PinnedRead,
};

use super::request::{
    BUSY, CANCELLED, CAPACITY, INTERNAL, STALE, STORAGE, TIMED_OUT, VERSION_MISMATCH,
};
use super::stops::{authorization, decode, native, principal_resolution, session, work};
use super::summary::{WorthQueryRefusalSummary as Summary, WorthQueryUserFacingRefusal};

impl WorthQueryUserFacingRefusal for WorthQueryWorkflowObservationDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::ProductSelection(_) => BUSY,
            Self::Read(cause) => workflow_read(cause),
        }
    }
}

/// A workflow page's own stop. Withheld actor roles are not refusals.
pub(super) fn workflow_read(cause: &WorkflowRead) -> Summary {
    match cause {
        WorkflowRead::Authorization(denial) => authorization(&denial.kind()),
        WorkflowRead::Principal(denial) => principal_resolution(denial.kind()),
        WorkflowRead::Work(cause) => work(cause),
        WorkflowRead::Relational(cause) => native(cause),
        WorkflowRead::Page(cause) => page(cause),
        WorkflowRead::PageShape(cause) => ordered(cause.kind()),
        WorkflowRead::Index(kind) => field(*kind),
        WorkflowRead::Reconstruction(cause) => maintenance(&cause.kind),
        WorkflowRead::PinnedRead(cause) => pinned(cause),
        WorkflowRead::Preparation(stop) => preflight(stop),
        WorkflowRead::Execution(cause) => attempt(cause),
        WorkflowRead::ContinuationMismatch => STALE,
        WorkflowRead::Corrupt => STORAGE,
        WorkflowRead::UnsupportedInstanceProtocol(_) => VERSION_MISMATCH,
        WorkflowRead::SubscriptionCapacity | WorkflowRead::Allocation(_) => CAPACITY,
        WorkflowRead::SelectedIndexUnavailable | WorkflowRead::BasisUnavailable => BUSY,
        WorkflowRead::PolicyMismatch
        | WorkflowRead::Installation(_)
        | WorkflowRead::Phase(_)
        | WorkflowRead::Inspection(_) => INTERNAL,
    }
}

/// The subject page's bounded prefix lookup.
pub(super) fn page(cause: &Page) -> Summary {
    match cause {
        Page::Work(cause) => work(cause),
        Page::Allocation(_) | Page::Memory(_) => CAPACITY,
        Page::Read(cause) => native(cause),
        Page::Lookup(cause) => ordered(cause.kind()),
        Page::FieldLookup(cause) => field(cause.kind()),
        Page::ExactBasisRequired => BUSY,
        Page::InvalidPrefix | Page::ForeignBoundary | Page::AccountingOverflow => INTERNAL,
    }
}

fn ordered(kind: OrderedLookup) -> Summary {
    match kind {
        OrderedLookup::Execution(cause) => native(&cause),
        OrderedLookup::CorruptIndexEntries | OrderedLookup::StorageParityMismatch => STORAGE,
        OrderedLookup::SnapshotUnavailable
        | OrderedLookup::ExactGenerationUnavailable
        | OrderedLookup::ExpectedGenerationMismatch => BUSY,
        OrderedLookup::InvalidPageWidth
        | OrderedLookup::IndexNotInstalled
        | OrderedLookup::WrongIndexKind
        | OrderedLookup::ForeignBoundary => INTERNAL,
    }
}

fn field(kind: FieldLookup) -> Summary {
    match kind {
        FieldLookup::Execution(cause) => native(&cause),
        FieldLookup::CorruptIndexEntries | FieldLookup::StorageParityMismatch => STORAGE,
        FieldLookup::SnapshotUnavailable | FieldLookup::ExactGenerationUnavailable => BUSY,
        FieldLookup::InvalidCandidateLimit
        | FieldLookup::IndexNotInstalled
        | FieldLookup::WrongIndexKind => INTERNAL,
    }
}

fn maintenance(kind: &Maintenance) -> Summary {
    match kind {
        Maintenance::Execution(cause) => native(cause),
        Maintenance::WorkBudgetExceeded
        | Maintenance::ColdRecordSlotBudgetExceeded
        | Maintenance::GenerationIdentityExhausted => CAPACITY,
        Maintenance::Basis(_)
        | Maintenance::SnapshotUnavailable
        | Maintenance::IndexUnavailable(_)
        | Maintenance::ColdReconstructionRequired
        | Maintenance::CandidateUnavailable => BUSY,
        Maintenance::CommitMismatch
        | Maintenance::BeforeRootMismatch
        | Maintenance::GenerationKindMismatch(_)
        | Maintenance::PriorEntryMismatch
        | Maintenance::ForeignCandidate
        | Maintenance::CandidateIndexesAlreadyPrepared => INTERNAL,
    }
}

fn pinned(cause: &PinnedRead) -> Summary {
    match cause {
        PinnedRead::Request(cause) => native(cause),
        PinnedRead::SnapshotUnavailable => BUSY,
        PinnedRead::IndexNotInstalled(_) => INTERNAL,
    }
}

fn preflight(stop: &Preflight) -> Summary {
    match stop {
        Preflight::Execution(cause) => native(cause),
        Preflight::Interrupted(event) => match event.interruption() {
            Interruption::Cancelled => CANCELLED,
            Interruption::TimedOut => TIMED_OUT,
        },
        Preflight::WorkCounterOverflow | Preflight::PreparationMemoryCounterOverflow => CAPACITY,
        Preflight::TopologyPending
        | Preflight::SelectedPositionUnavailable { .. }
        | Preflight::ManagedSourceNotPrepared
        | Preflight::ManagedSourceChanged
        | Preflight::RegistrationChanged => BUSY,
        Preflight::SelectedSourceMismatch | Preflight::ForeignCell => INTERNAL,
    }
}

/// The workflow read's own native execution, by its attempt kind.
pub(super) fn attempt(cause: &WorthQueryApplicationAttemptDenial) -> Summary {
    match cause.kind() {
        Attempt::Cancelled => CANCELLED,
        Attempt::DeadlineExceeded => TIMED_OUT,
        Attempt::EffectExecution(cause) => work(&cause),
        Attempt::ExecutionRequest(kind) => session(&kind),
        Attempt::ValueDecode(cause) => decode(&cause),
        Attempt::StorageUnavailable
        | Attempt::ContentInconsistency
        | Attempt::RetainedSourceStorageDenied => STORAGE,
        Attempt::AllocationDenied
        | Attempt::DecisionFactBudgetExceeded
        | Attempt::CandidateCapacityExceeded => CAPACITY,
        Attempt::UniqueIndexUnavailable => BUSY,
        // Every other kind is a write-path or installation defect a read
        // cannot cause by itself.
        _ => INTERNAL,
    }
}

#[cfg(test)]
#[path = "workflow_read_tests.rs"]
mod tests;
