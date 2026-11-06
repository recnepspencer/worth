//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    #[doc(hidden)]
    pub fn mapping_entity_id_for_test(&self) -> EntityId {
        Self::mapping_entity_id(self)
    }
}

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    #[doc(hidden)]
    pub fn principal_entity_id_for_test(&self) -> EntityId {
        Self::principal_entity_id(self)
    }
}

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    #[doc(hidden)]
    pub fn target_relation_id_for_test(&self) -> RelationId {
        Self::target_relation_id(self)
    }
}
