//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn entity_for_binding_role_for_test<
        Binding: 'static,
    >(
        &self, role: &str) -> Result<EntityId, WorthQueryApplicationOutputProjectionDenial> {
        Self::entity_for_binding_role::<Binding>(self, role)
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn native_witness_roles_for_test(
        &self) -> impl ExactSizeIterator<Item = (&str, WorthQueryApplicationOutputPosture, &str, EntityId)> + Clone {
        Self::native_witness_roles(self)
    }
}
