//! Retained fact and key ownership under the selected allocation policy.
use std::cell::RefCell;
use worth_execution::{ExecutionAllocationDenial, ExecutionArray};

mod canonical_key;
mod control;
mod directory;
pub(in crate::domain_computation::primary_graph) mod endpoints;
mod fact_store;
#[cfg(test)]
mod handoff;
mod run;
mod storage;
mod streaming;
#[cfg(test)]
mod tests;

pub(in crate::domain_computation::primary_graph) use canonical_key::AdmittedFactKey;
pub(in crate::domain_computation::primary_graph) use control::StorageControl;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use handoff::restore_authored_order;
pub(in crate::domain_computation::primary_graph) use storage::RetainedFactStore;

type Slot<T> = RefCell<Option<RetainedFact<T>>>;

/// Algorithmic author/merge granule, not an admission or request count limit.
const AUTHOR_CHUNK: usize = 32;

pub(in crate::domain_computation::primary_graph) struct RetainedFact<T> {
    pub(in crate::domain_computation::primary_graph) key: AdmittedFactKey,
    pub(in crate::domain_computation::primary_graph) value: T,
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) ordinal: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum StoreDenial {
    Allocation(ExecutionAllocationDenial),
    RequestInterruption(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    ),
    Representability,
    ConflictingBody,
    #[cfg(test)]
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

pub(in crate::domain_computation::primary_graph) use fact_store::{
    AuthoringSourceFacts, RetainedSourceFactValues, RetainedSourceFacts,
};

impl StoreDenial {
    pub(in crate::domain_computation::primary_graph) fn into_attempt_denial(
        self,
        subject: &str,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial {
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationAttemptDenial as Denial,
            WorthQueryApplicationAttemptDenialKind as Kind,
        };
        let kind = match &self {
            Self::Allocation(_) => Kind::AllocationDenied,
            Self::RequestInterruption(_) => Kind::CurrentAuthorityDenied,
            Self::ConflictingBody => Kind::DecisionDependencyMismatch,
            _ => Kind::RetainedSourceStorageDenied,
        };
        Denial::retained_source_denied(kind, subject, self)
    }
}
