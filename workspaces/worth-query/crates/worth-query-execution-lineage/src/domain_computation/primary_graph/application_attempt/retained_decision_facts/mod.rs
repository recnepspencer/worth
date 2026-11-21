//! Retained fact and key ownership under the selected allocation policy.
use std::cell::RefCell;
use worth_execution::{ExecutionAllocationDenial, ExecutionArray};

pub mod canonical_key;
pub mod control;
pub mod directory;
pub mod endpoints;
pub mod fact_store;
pub mod handoff;
pub mod run;
pub mod storage;
pub mod streaming;

pub use canonical_key::AdmittedFactKey;
pub use control::StorageControl;

pub use storage::RetainedFactStore;

pub type Slot<T> = RefCell<Option<RetainedFact<T>>>;

/// Algorithmic author/merge granule, not an admission or request count limit.
pub const AUTHOR_CHUNK: usize = 32;

pub struct RetainedFact<T> {
    pub key: AdmittedFactKey,
    pub value: T,
    pub ordinal: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreDenial {
    Allocation(ExecutionAllocationDenial),
    ExecutionRequest(worth_execution::WorkCeilingDenial),
    RequestInterruption(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    ),
    Representability,
    PhysicalAllocation,
    ConflictingBody,
    ContentInconsistency,
    Content(worth_relational::facade::content::RelationalContentDenial),
    InvalidOrdinal,
}

impl From<ExecutionAllocationDenial> for StoreDenial {
    fn from(denial: ExecutionAllocationDenial) -> Self {
        Self::Allocation(denial)
    }
}

/// Layout covers inline slots only; T's nested owned heaps need their own owner.
type Slots<T> = ExecutionArray<Slot<T>>;

impl std::fmt::Display for StoreDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "retained fact storage refused: {self:?}")
    }
}
impl std::error::Error for StoreDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(denial) => Some(denial),
            _ => None,
        }
    }
}

pub use fact_store::{
    AuthoringSourceFacts, RetainedSourceFactValues, RetainedSourceFacts,
};

impl StoreDenial {
    pub fn into_attempt_denial(
        self,
        subject: &str,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial {
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationAttemptDenial as Denial,
            WorthQueryApplicationAttemptDenialKind as Kind,
        };
        if let Self::ExecutionRequest(cause) = self {
            return Denial::relational_read_stopped(cause.into(), subject);
        }
        if let Self::Content(cause) = self {
            return Denial::relational_read_stopped(cause.into_execution_cause(), subject);
        }
        let kind = match &self {
            Self::Allocation(_) | Self::PhysicalAllocation => Kind::AllocationDenied,
            Self::RequestInterruption(_) => Kind::CurrentAuthorityDenied,
            Self::ExecutionRequest(_) => Kind::RetainedSourceStorageDenied,
            Self::ConflictingBody => Kind::DecisionDependencyMismatch,
            Self::ContentInconsistency => Kind::ContentInconsistency,
            _ => Kind::RetainedSourceStorageDenied,
        };
        Denial::retained_source_denied(kind, subject, self)
    }
}

impl worth_execution::ChargedBytes for StoreDenial {
    fn additional_charged_bytes(&self) -> u64 {
        // Every current variant contains only inline denial/control data.
        match self {
            Self::Allocation(denial) => {
                worth_execution::ChargedBytes::additional_charged_bytes(denial)
            }
            Self::RequestInterruption(interruption) => {
                fn inline<T: Copy>(_: &T) {}
                inline(interruption);
                0
            }
            Self::ExecutionRequest(cause) => {
                fn inline<T: Copy>(_: &T) {}
                inline(cause);
                0
            }
            Self::Representability
            | Self::ConflictingBody
            | Self::PhysicalAllocation
            | Self::ContentInconsistency
            | Self::Content(_) => 0,
            Self::InvalidOrdinal => 0,
        }
    }
}

mod court_doors_for_test;
