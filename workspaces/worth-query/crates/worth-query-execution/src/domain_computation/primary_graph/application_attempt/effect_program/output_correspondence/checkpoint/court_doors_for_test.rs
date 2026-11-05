//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputCorrespondence {
    #[doc(hidden)]
    pub fn from_checkpoint_roles_for_test<'a>(
        binding_type: TypeId,
        contract_type: TypeId,
        optional_roles: impl Iterator<Item = &'a str> + Clone,
        roles: &[WorthQueryCheckpointOutputRole],
        entity_type: impl FnMut(&str) -> Option<TypeId>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        Self::from_checkpoint_roles(
            binding_type,
            contract_type,
            optional_roles,
            roles,
            entity_type,
            execution,
        )
    }
}

impl WorthQueryApplicationOutputCorrespondence {
    /// Round-trip this real live correspondence through the checkpoint role owner.
    /// Returns only equality; no reconstructed output or custody escapes.
    #[doc(hidden)]
    pub fn checkpoint_roundtrip_equal_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, CheckpointDecodeDenial> {
        let missing = || {
            CheckpointDecodeDenial::from("live correspondence has no concrete contract".to_owned())
        };
        let rebound = Self::from_checkpoint_roles(
            self.binding_type.ok_or_else(missing)?,
            self.contract_type.ok_or_else(missing)?,
            self.storage.optional_roles.iter().map(String::as_str),
            &self.checkpoint_roles(),
            |entity| {
                self.storage
                    .roles
                    .iter()
                    .find(|(_, r)| r.entity_name == entity)
                    .map(|(_, r)| r.entity_type)
            },
            execution,
        )?;
        Ok(self == &rebound)
    }
}
