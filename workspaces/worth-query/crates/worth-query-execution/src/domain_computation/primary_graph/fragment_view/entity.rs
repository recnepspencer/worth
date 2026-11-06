//! Erasure preserves each original native read or effect owner's affinity.

use super::super::{WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget};
use std::marker::PhantomData;
use std::sync::Arc;
use worth_relational::facade::transactions::EntityReference;

pub(in crate::domain_computation::primary_graph) struct ErasedObserved {
    pub(in crate::domain_computation::primary_graph) identity:
        WorthQueryInvariantEntityIdentity<(), ()>,
}
pub(in crate::domain_computation::primary_graph) struct ErasedCandidate {
    pub(in crate::domain_computation::primary_graph) reference: EntityReference,
    pub(in crate::domain_computation::primary_graph) entity: String,
    pub(in crate::domain_computation::primary_graph) created_effect: Option<usize>,
    pub(in crate::domain_computation::primary_graph) program: Arc<()>,
}

pub struct ObservedEntity<E> {
    pub(in crate::domain_computation::primary_graph) erased: ErasedObserved,
    pub(in crate::domain_computation::primary_graph) marker: PhantomData<fn() -> E>,
}
pub struct CandidateEntity<E> {
    pub(in crate::domain_computation::primary_graph) erased: ErasedCandidate,
    pub(in crate::domain_computation::primary_graph) marker: PhantomData<fn() -> E>,
}
pub struct ObservedTarget<E> {
    pub(in crate::domain_computation::primary_graph) erased:
        WorthQueryInvariantMutationTarget<(), ()>,
    pub(in crate::domain_computation::primary_graph) marker: PhantomData<fn() -> E>,
}
