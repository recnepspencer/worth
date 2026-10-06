//! Compiler-visible invalidation phase truth.
//!
//! Algorithms remain with their runtime owners. These wrappers make it
//! impossible to mistake prepared intent for performed publication.

// Phase 2 seals the complete owner progression before the Phase 4 scheduling
// cutover makes every form hot. Until then, several later-phase forms are
// intentionally exercised only by the compiler and contract tests below.
#![allow(dead_code, unused_imports)]

mod binding;
mod committed;
mod denial;
mod executed;
mod graph_batch;
mod lowered;
mod owner;
mod prepared;
mod ready;
mod resolved;
mod source;
mod structural;

pub(crate) use binding::{
    InvalidationOriginBinding, InvalidationOriginBindingAxes, InvalidationReadinessEpoch,
    InvalidationStageOrder, InvalidationWorkBindingAxes,
};
pub(crate) use committed::{AdmittedDependencyRecompute, CommittedDirectInvalidation};
pub(crate) use denial::{InvalidationOriginAdmissionOutcome, InvalidationProgressionDenial};
pub(crate) use executed::ExecutedInvalidationBatch;
pub(crate) use graph_batch::{
    DisjointGraphBatch, GraphBatchBindingAxes, GraphMemberBinding, GraphProposalKey,
};
pub(crate) use lowered::LoweredInvalidationBatch;
pub(crate) use owner::InvalidationProgressionOwner;
pub(crate) use prepared::PreparedDirectInvalidation;
pub(crate) use ready::ReadyInvalidationBatch;
pub(crate) use resolved::{InvalidationWorkBatch, InvalidationWorkItem, ResolvedInvalidationWork};
pub(crate) use source::AdmittedSourceRecompute;
pub(crate) use structural::AdmittedStructuralRecompute;

#[cfg(test)]
mod tests;
