//! Erasure preserves each original native read or effect owner's affinity.

use super::super::{WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget};
use std::marker::PhantomData;
use std::sync::Arc;
use worth_relational::facade::transactions::EntityReference;

pub struct ErasedObserved {
    pub identity: WorthQueryInvariantEntityIdentity<(), ()>,
}
pub struct ErasedCandidate {
    pub reference: EntityReference,
    pub entity: String,
    pub created_effect: Option<usize>,
    pub program: Arc<()>,
}

pub struct ObservedEntity<E> {
    pub(crate) erased: ErasedObserved,
    pub(crate) marker: PhantomData<fn() -> E>,
}
pub struct CandidateEntity<E> {
    pub(crate) erased: ErasedCandidate,
    pub(crate) marker: PhantomData<fn() -> E>,
}
pub struct ObservedTarget<E> {
    pub(crate) erased: WorthQueryInvariantMutationTarget<(), ()>,
    pub(crate) marker: PhantomData<fn() -> E>,
}

/// Crate-boundary internal ports for fragment-view entity handles.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait FragmentObservedEntityInternalPort<E>: Sized {
    fn from_erased(erased: ErasedObserved) -> Self;
    fn erased_ref(&self) -> &ErasedObserved;
}

impl<E> FragmentObservedEntityInternalPort<E> for ObservedEntity<E> {
    fn from_erased(erased: ErasedObserved) -> Self {
        Self {
            erased,
            marker: PhantomData,
        }
    }

    fn erased_ref(&self) -> &ErasedObserved {
        &self.erased
    }
}

pub trait FragmentCandidateEntityInternalPort<E>: Sized {
    fn from_erased(erased: ErasedCandidate) -> Self;
    fn erased_ref(&self) -> &ErasedCandidate;
}

impl<E> FragmentCandidateEntityInternalPort<E> for CandidateEntity<E> {
    fn from_erased(erased: ErasedCandidate) -> Self {
        Self {
            erased,
            marker: PhantomData,
        }
    }

    fn erased_ref(&self) -> &ErasedCandidate {
        &self.erased
    }
}

pub trait FragmentObservedTargetInternalPort<E>: Sized {
    fn from_erased(erased: WorthQueryInvariantMutationTarget<(), ()>) -> Self;
    fn erased_ref(&self) -> &WorthQueryInvariantMutationTarget<(), ()>;
}

impl<E> FragmentObservedTargetInternalPort<E> for ObservedTarget<E> {
    fn from_erased(erased: WorthQueryInvariantMutationTarget<(), ()>) -> Self {
        Self {
            erased,
            marker: PhantomData,
        }
    }

    fn erased_ref(&self) -> &WorthQueryInvariantMutationTarget<(), ()> {
        &self.erased
    }
}
