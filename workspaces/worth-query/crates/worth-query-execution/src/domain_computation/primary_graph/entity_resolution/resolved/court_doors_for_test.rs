//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn entity_id_for_test(&self) -> EntityId {
        Self::entity_id(self)
    }
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self) -> KindId {
        Self::entity_kind(self)
    }
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn identity_value_for_test(&self) -> &AspectValue {
        Self::identity_value(self)
    }
}

impl WorthQueryResolvedEntity {
    #[doc(hidden)]
    pub fn into_application_identity_for_test<Schema, Entity>(
        self,
    ) -> WorthQueryApplicationEntityIdentity<Schema, Entity> {
        Self::into_application_identity::<Schema, Entity>(self)
    }
}

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn identity_index_id_for_test(&self) -> DerivedIndexId {
        Self::identity_index_id(self)
    }
}
