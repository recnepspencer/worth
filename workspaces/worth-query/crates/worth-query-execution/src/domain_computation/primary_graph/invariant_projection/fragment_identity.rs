//! Schema erasure retains the exact projection and operation affinities.

use super::super::fragment_view::{ErasedObserved, ObservedEntity};
use super::{WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget};
use std::marker::PhantomData;
use std::sync::Arc;

impl<S, E> WorthQueryInvariantEntityIdentity<S, E> {
    pub fn for_fragment(&self) -> ObservedEntity<E> {
        ObservedEntity {
            erased: ErasedObserved {
                identity: self.fragment_schema::<()>(),
            },
            marker: PhantomData,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn fragment_schema<Schema>(
        &self,
    ) -> WorthQueryInvariantEntityIdentity<Schema, ()> {
        WorthQueryInvariantEntityIdentity {
            entity_id: self.entity_id,
            kind: self.kind,
            entity: Arc::clone(&self.entity),
            authority_identity: self.authority_identity,
            _marker: PhantomData,
        }
    }
}

impl<S, E> WorthQueryInvariantMutationTarget<S, E> {
    pub(in crate::domain_computation::primary_graph) fn fragment_schema<Schema>(
        &self,
    ) -> WorthQueryInvariantMutationTarget<Schema, ()> {
        WorthQueryInvariantMutationTarget {
            entity_id: self.entity_id,
            entity: Arc::clone(&self.entity),
            runtime_authority: self.runtime_authority,
            binding_identity: self.binding_identity.clone(),
            admission_identity: self.admission_identity,
            _marker: PhantomData,
        }
    }
}
