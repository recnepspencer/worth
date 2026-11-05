//! Moved whole from `domain_computation::authorization::operation_scope_binding`: the items below and every impl they own.
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

/// Exact identity of one entity (partition, slot, and generation) that an
/// admitted operation was bound to as its principal or scope.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryOperationScopeEntityBinding {
    pub(in crate::domain_computation) partition_id: u32,
    pub(in crate::domain_computation) local_slot: u64,
    pub(in crate::domain_computation) generation: u32,
}

impl WorthQueryOperationScopeEntityBinding {
    pub(crate) fn from_entity(entity: worth_relational::facade::identity::EntityId) -> Self {
        Self {
            partition_id: entity.partition_value(),
            local_slot: entity.local_slot_value(),
            generation: entity.generation_value(),
        }
    }

    pub const fn partition_id(self) -> u32 {
        self.partition_id
    }

    pub const fn local_slot(self) -> u64 {
        self.local_slot
    }

    pub const fn generation(self) -> u32 {
        self.generation
    }
}

/// Fixed descriptive affinity used only to compare completed producer
/// decisions. The admitted operation remains the authority to execute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDecisionScopeWitness {
    pub(in crate::domain_computation) runtime_authority: u64,
    pub(in crate::domain_computation) binding_identity: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation) operation_authority_identity: [u8; 32],
    pub(in crate::domain_computation) principal: WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation) scope: WorthQueryOperationScopeEntityBinding,
}

impl WorthQueryOperationScopeEntityBinding {
    #[doc(hidden)]
    pub fn from_entity_for_test(entity: worth_relational::facade::identity::EntityId) -> Self {
        Self::from_entity(entity)
    }
}
