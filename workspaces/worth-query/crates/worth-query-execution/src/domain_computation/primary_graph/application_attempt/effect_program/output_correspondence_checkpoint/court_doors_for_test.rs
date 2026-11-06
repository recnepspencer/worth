//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryCheckpointOutputRole;

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
