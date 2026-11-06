//! Sealed test doors over this module's private members.
use super::*;

pub trait OutputCorrespondenceRestorationDoorsOps: Sized {
    #[doc(hidden)]
    fn from_checkpoint_roles_for_test<'a>(
        binding_type: TypeId,
        contract_type: TypeId,
        optional_roles: impl Iterator<Item = &'a str> + Clone,
        roles: &[WorthQueryCheckpointOutputRole],
        entity_type: impl FnMut(&str) -> Option<TypeId>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial>;
}

impl OutputCorrespondenceRestorationDoorsOps for WorthQueryApplicationOutputCorrespondence {
    fn from_checkpoint_roles_for_test<'a>(
        binding_type: TypeId,
        contract_type: TypeId,
        optional_roles: impl Iterator<Item = &'a str> + Clone,
        roles: &[WorthQueryCheckpointOutputRole],
        entity_type: impl FnMut(&str) -> Option<TypeId>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        <Self as OutputCorrespondenceRestoration>::from_checkpoint_roles(
            binding_type,
            contract_type,
            optional_roles,
            roles,
            entity_type,
            execution,
        )
    }
}
