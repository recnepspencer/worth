//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantEntityIdentity;

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn entity_for_test(&self) -> &Arc<str> {
        &self.entity
    }
}

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn entity_id_for_test(&self) -> &EntityId {
        &self.entity_id
    }
}

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn new_for_test(
        entity_id: EntityId,
        kind: KindId,
        entity: Arc<str>,
        authority_identity: u64,
        _marker: PhantomData<fn() -> (Schema, Entity)>,
    ) -> Self {
        Self {
            entity_id,
            kind,
            entity,
            authority_identity,
            _marker,
        }
    }
}
