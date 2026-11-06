//! Retained fact and key ownership under the selected allocation policy.
use std::cell::RefCell;
use worth_execution::ExecutionArray;

pub(crate) mod canonical_key;
pub(crate) mod directory;
pub(crate) mod fact_store;
pub(crate) mod handoff;
pub(crate) mod run;
pub(crate) mod storage;
mod streaming;

pub(in crate::domain_computation::primary_graph) use canonical_key::AdmittedFactKey;

pub(in crate::domain_computation::primary_graph) use storage::RetainedFactStore;

pub type Slot<T> = RefCell<Option<RetainedFact<T>>>;

/// Algorithmic author/merge granule, not an admission or request count limit.
pub const AUTHOR_CHUNK: usize = 32;

pub struct RetainedFact<T> {
    pub(in crate::domain_computation::primary_graph) key: AdmittedFactKey,
    pub(in crate::domain_computation::primary_graph) value: T,
    pub(in crate::domain_computation::primary_graph) ordinal: usize,
}

/// Layout covers inline slots only; T's nested owned heaps need their own owner.
type Slots<T> = ExecutionArray<Slot<T>>;

pub(in crate::domain_computation::primary_graph) use fact_store::{
    AuthoringSourceFacts, RetainedSourceFactValues, RetainedSourceFacts,
};

mod court_doors_for_test;
