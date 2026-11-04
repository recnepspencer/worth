//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn entity_for_binding_role_for_test<Binding: 'static>(
        &self,
        role: &str,
    ) -> Result<EntityId, WorthQueryApplicationOutputProjectionDenial> {
        Self::entity_for_binding_role::<Binding>(self, role)
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn native_witness_roles_for_test(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, WorthQueryApplicationOutputPosture, &str, EntityId)> + Clone
    {
        Self::native_witness_roles(self)
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn bound_entity_for_test(
        &self,
        role: &OutputRoleUse,
    ) -> Result<Option<EntityId>, WorthQueryApplicationOutputProjectionDenial> {
        Self::bound_entity(self, role)
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn outputs_of_for_test<Contract: 'static>(
        &self,
    ) -> Result<
        WorthQueryApplicationTypedOutputCorrespondence<'_, Contract>,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        Self::outputs_of::<Contract>(self)
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn workflow_content_identity_for_test(&self) -> [u8; 32] {
        Self::workflow_content_identity(self)
    }
}

impl WorthQueryCheckpointOutputRole {
    #[doc(hidden)]
    pub fn entity_mut_for_test(&mut self) -> &mut EntityId {
        &mut self.entity
    }
}

impl WorthQueryCheckpointOutputRole {
    #[doc(hidden)]
    pub fn entity_name_mut_for_test(&mut self) -> &mut String {
        &mut self.entity_name
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn family_members_for_test<Family>(
        &self,
    ) -> Result<
        impl Iterator<
            Item = Result<
                (&str, WorthQueryApplicationOutputPosture, EntityId),
                WorthQueryApplicationOutputProjectionDenial,
            >,
        > + '_,
        WorthQueryApplicationOutputProjectionDenial,
    >
    where
        Family: WorthQueryApplicationOutputRoleFamily,
    {
        Self::family_members::<Family>(self)
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn active_entity_for_role_for_test(&self, role: &str) -> Option<EntityId> {
        Self::active_entity_for_role(self, role)
    }
}

impl WorthQueryCheckpointOutputRole {
    #[doc(hidden)]
    pub fn new_for_test(
        role: String,
        posture: WorthQueryApplicationOutputPosture,
        entity_name: String,
        entity: EntityId,
    ) -> Self {
        Self {
            role,
            posture,
            entity_name,
            entity,
        }
    }
}
