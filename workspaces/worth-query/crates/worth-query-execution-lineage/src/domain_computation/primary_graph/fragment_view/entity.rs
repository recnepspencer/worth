//! Erasure preserves each original native read or effect owner's affinity.

use super::super::{WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget};
use std::marker::PhantomData;
use std::sync::Arc;
use worth_relational::facade::transactions::EntityReference;

pub struct ErasedObserved {
    pub identity:
        WorthQueryInvariantEntityIdentity<(), ()>,
}
pub struct ErasedCandidate {
    pub reference: EntityReference,
    pub entity: String,
    pub created_effect: Option<usize>,
    pub program: Arc<()>,
}

pub struct ObservedEntity<E> {
    pub erased: ErasedObserved,
    pub marker: PhantomData<fn() -> E>,
}
pub struct CandidateEntity<E> {
    pub erased: ErasedCandidate,
    pub marker: PhantomData<fn() -> E>,
}
pub struct ObservedTarget<E> {
    pub erased:
        WorthQueryInvariantMutationTarget<(), ()>,
    pub marker: PhantomData<fn() -> E>,
}
