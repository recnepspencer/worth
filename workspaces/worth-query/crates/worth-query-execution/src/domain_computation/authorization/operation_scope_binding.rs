//! Stable typed binding between an operation admission and its exact scope.

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

/// Exact identity of one entity (partition, slot, and generation) that an
/// admitted operation was bound to as its principal or scope.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryOperationScopeEntityBinding {
    partition_id: u32,
    local_slot: u64,
    generation: u32,
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

/// The runtime, installed schema binding, operation, principal, and scope that
/// one operation admission was bound to.
///
/// Carried on commit receipts, recovery handles, and invariant contexts so later
/// steps can compare against the exact admitted binding. It is descriptive and
/// grants nothing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationScopeBinding {
    runtime_authority: u64,
    binding_identity: ApplicationSchemaBindingIdentity,
    operation_authority_identity: Arc<str>,
    principal: WorthQueryOperationScopeEntityBinding,
    scope: WorthQueryOperationScopeEntityBinding,
}

/// Fixed descriptive affinity used only to compare completed producer
/// decisions. The admitted operation remains the authority to execute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryDecisionScopeWitness {
    runtime_authority: u64,
    binding_identity: ApplicationSchemaBindingIdentity,
    operation_authority_identity: [u8; 32],
    principal: WorthQueryOperationScopeEntityBinding,
    scope: WorthQueryOperationScopeEntityBinding,
}

impl WorthQueryOperationScopeBinding {
    pub(in crate::domain_computation) fn decision_reuse_witness(
        &self,
        operation_authority_identity: [u8; 32],
    ) -> WorthQueryDecisionScopeWitness {
        WorthQueryDecisionScopeWitness {
            runtime_authority: self.runtime_authority,
            binding_identity: self.binding_identity.clone(),
            operation_authority_identity,
            principal: self.principal,
            scope: self.scope,
        }
    }

    pub(super) fn mint(
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
        operation_authority_identity: &str,
        principal: worth_relational::facade::identity::EntityId,
        scope: worth_relational::facade::identity::EntityId,
    ) -> Self {
        Self {
            runtime_authority: runtime_authority.as_u64(),
            binding_identity: binding_identity.clone(),
            operation_authority_identity: Arc::from(operation_authority_identity),
            principal: WorthQueryOperationScopeEntityBinding::from_entity(principal),
            scope: WorthQueryOperationScopeEntityBinding::from_entity(scope),
        }
    }

    pub const fn runtime_authority(&self) -> u64 {
        self.runtime_authority
    }

    pub const fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn operation_authority_identity(&self) -> &str {
        &self.operation_authority_identity
    }

    pub const fn principal(&self) -> WorthQueryOperationScopeEntityBinding {
        self.principal
    }

    pub const fn scope(&self) -> WorthQueryOperationScopeEntityBinding {
        self.scope
    }

    /// Raw scope parts for per-axis drift proof (R8.28). Not production.
    #[cfg(test)]
    pub(crate) fn axis_probe_scope(
        runtime_authority: u64,
        binding_identity: ApplicationSchemaBindingIdentity,
        operation_authority_identity: &str,
        principal_partition: u32,
        principal_slot: u64,
        principal_generation: u32,
        scope_partition: u32,
        scope_slot: u64,
        scope_generation: u32,
    ) -> Self {
        Self {
            runtime_authority,
            binding_identity,
            operation_authority_identity: Arc::from(operation_authority_identity),
            principal: WorthQueryOperationScopeEntityBinding {
                partition_id: principal_partition,
                local_slot: principal_slot,
                generation: principal_generation,
            },
            scope: WorthQueryOperationScopeEntityBinding {
                partition_id: scope_partition,
                local_slot: scope_slot,
                generation: scope_generation,
            },
        }
    }
}
