use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_operation::{
    WorthQueryApplicationOutputRoleFamily, WorthQueryApplicationOutputRoleNameDenial,
};
use worth_relational::facade::identity::EntityId;

pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;

mod candidate;
pub(in crate::domain_computation::primary_graph::application_attempt) use candidate::WorthQueryApplicationOutputCorrespondenceCandidate;

mod family_entry;
pub use family_entry::WorthQueryApplicationOutputFamilyEntry;

mod projection;
pub use projection::{
    WorthQueryApplicationOutputEntity, WorthQueryApplicationOutputProjectionDenial,
};

mod role_use;
pub(in crate::domain_computation::primary_graph) use role_use::OutputRoleUse;

mod typed;
pub use typed::WorthQueryApplicationTypedOutputCorrespondence;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommittedOutputBinding {
    posture: WorthQueryApplicationOutputPosture,
    entity_name: String,
    entity_type: TypeId,
    entity: EntityId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCheckpointOutputRole {
    pub(in crate::domain_computation::primary_graph) role: String,
    pub(in crate::domain_computation::primary_graph) posture: WorthQueryApplicationOutputPosture,
    pub(in crate::domain_computation::primary_graph) entity_name: String,
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
}

/// Sealed role-to-identity correspondence resolved from one Relational commit.
///
/// It is stored erased, so receipts and settlements can be cloned, replayed
/// and readmitted without their contract type. Consumers read it only through
/// [`WorthQueryApplicationTypedOutputCorrespondence`], which checks the
/// contract once and then checks every role read at compile time.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationOutputCorrespondence {
    binding_type: Option<TypeId>,
    contract_type: Option<TypeId>,
    optional_roles: BTreeSet<String>,
    roles: BTreeMap<String, CommittedOutputBinding>,
}

impl WorthQueryApplicationOutputCorrespondence {
    pub(in crate::domain_computation::primary_graph) fn from_checkpoint_roles(
        binding_type: TypeId,
        contract_type: TypeId,
        optional_roles: BTreeSet<String>,
        roles: Vec<WorthQueryCheckpointOutputRole>,
        mut entity_type: impl FnMut(&str) -> Option<TypeId>,
    ) -> Result<Self, String> {
        let mut rebound = BTreeMap::new();
        for role in roles {
            WorthQueryApplicationOutputRoleNameDenial::validate(&role.role)
                .map_err(|denial| format!("checkpoint output role {}: {denial}", role.role))?;
            let marker = entity_type(&role.entity_name).ok_or_else(|| {
                format!(
                    "checkpoint output role {} names an uninstalled entity {}",
                    role.role, role.entity_name
                )
            })?;
            let binding = CommittedOutputBinding {
                posture: role.posture,
                entity_name: role.entity_name,
                entity_type: marker,
                entity: role.entity,
            };
            if rebound.insert(role.role.clone(), binding).is_some() {
                return Err(format!(
                    "checkpoint output role {} is duplicated",
                    role.role
                ));
            }
        }
        Ok(Self {
            binding_type: Some(binding_type),
            contract_type: Some(contract_type),
            optional_roles,
            roles: rebound,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn checkpoint_roles(
        &self,
    ) -> Vec<WorthQueryCheckpointOutputRole> {
        self.roles
            .iter()
            .map(|(role, binding)| WorthQueryCheckpointOutputRole {
                role: role.clone(),
                posture: binding.posture,
                entity_name: binding.entity_name.clone(),
                entity: binding.entity,
            })
            .collect()
    }

    pub(in crate::domain_computation::primary_graph) fn workflow_content_identity(
        &self,
    ) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut digest = Sha256::new();
        digest.update(b"worth-query:workflow-assessment-output:v1");
        for (role, binding) in &self.roles {
            digest.update((role.len() as u64).to_le_bytes());
            digest.update(role.as_bytes());
            digest.update([match binding.posture {
                WorthQueryApplicationOutputPosture::Create => 0,
                WorthQueryApplicationOutputPosture::Preserve => 1,
                WorthQueryApplicationOutputPosture::Retire => 2,
            }]);
            digest.update(binding.entity.partition_value_u64().to_le_bytes());
            digest.update(binding.entity.local_slot_value().to_le_bytes());
            digest.update(u64::from(binding.entity.generation_value()).to_le_bytes());
        }
        digest.finalize().into()
    }

    /// This correspondence read as `Contract`, refused as
    /// [`WorthQueryApplicationOutputProjectionDenial::ForeignContract`] when
    /// the commit was made under another contract.
    pub(in crate::domain_computation::primary_graph) fn outputs_of<Contract: 'static>(
        &self,
    ) -> Result<
        WorthQueryApplicationTypedOutputCorrespondence<'_, Contract>,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        if self.contract_type != Some(TypeId::of::<Contract>()) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignContract);
        }
        Ok(WorthQueryApplicationTypedOutputCorrespondence::new(self))
    }

    pub(in crate::domain_computation::primary_graph) const fn binding_type(
        &self,
    ) -> Option<TypeId> {
        self.binding_type
    }

    pub(in crate::domain_computation::primary_graph) fn created_entity_ids(
        &self,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.roles.values().filter_map(|binding| {
            (binding.posture == WorthQueryApplicationOutputPosture::Create)
                .then_some(binding.entity)
        })
    }

    /// Only completed Retire bindings whose actual delete effects were validated.
    pub(in crate::domain_computation::primary_graph) fn retired_entity_ids(
        &self,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.roles.values().filter_map(|binding| {
            (binding.posture == WorthQueryApplicationOutputPosture::Retire)
                .then_some(binding.entity)
        })
    }

    /// Publication metadata includes retirement; it grants no current identity.
    pub(in crate::domain_computation::primary_graph) fn publication_entity_for_role(
        &self,
        role: &str,
    ) -> Option<EntityId> {
        self.roles.get(role).map(|binding| binding.entity)
    }

    pub(in crate::domain_computation::primary_graph) fn active_entity_for_role(
        &self,
        role: &str,
    ) -> Option<EntityId> {
        self.roles.get(role).and_then(|binding| {
            (binding.posture != WorthQueryApplicationOutputPosture::Retire)
                .then_some(binding.entity)
        })
    }

    /// The entity bound under `role`, or `None` when it is unbound, once the
    /// contract, the declared cardinality, the posture and the entity type
    /// agree with the use.
    pub(in crate::domain_computation::primary_graph) fn bound_entity(
        &self,
        role: &OutputRoleUse,
    ) -> Result<Option<EntityId>, WorthQueryApplicationOutputProjectionDenial> {
        if self.contract_type != Some(role.contract_type) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignContract);
        }
        if self.optional_roles.contains(&role.name) != role.cardinality.admits_absence() {
            return Err(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch);
        }
        let Some(binding) = self.roles.get(&role.name) else {
            return Ok(None);
        };
        if binding.posture != role.posture {
            return Err(WorthQueryApplicationOutputProjectionDenial::ActionMismatch);
        }
        if binding.entity_type != role.entity_type {
            return Err(WorthQueryApplicationOutputProjectionDenial::EntityMismatch);
        }
        Ok(Some(binding.entity))
    }

    pub(in crate::domain_computation::primary_graph) fn entity_for_binding_role<
        Binding: 'static,
    >(
        &self,
        role: &str,
    ) -> Result<EntityId, WorthQueryApplicationOutputProjectionDenial> {
        if self.binding_type != Some(TypeId::of::<Binding>()) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignBinding);
        }
        self.roles
            .get(role)
            .map(|binding| binding.entity)
            .ok_or(WorthQueryApplicationOutputProjectionDenial::MissingRole)
    }

    pub(in crate::domain_computation::primary_graph) fn posture_for_binding_role<
        Binding: 'static,
    >(
        &self,
        role: &str,
    ) -> Result<WorthQueryApplicationOutputPosture, WorthQueryApplicationOutputProjectionDenial>
    {
        if self.binding_type != Some(TypeId::of::<Binding>()) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignBinding);
        }
        self.roles
            .get(role)
            .map(|binding| binding.posture)
            .ok_or(WorthQueryApplicationOutputProjectionDenial::MissingRole)
    }

    pub(in crate::domain_computation::primary_graph) fn current_entity_for_role<Entity: 'static>(
        &self,
        role: &str,
    ) -> Result<Option<EntityId>, WorthQueryApplicationOutputProjectionDenial> {
        let Some(binding) = self.roles.get(role) else {
            return Ok(None);
        };
        if binding.entity_type != TypeId::of::<Entity>() {
            return Err(WorthQueryApplicationOutputProjectionDenial::EntityMismatch);
        }
        Ok(
            (binding.posture != WorthQueryApplicationOutputPosture::Retire)
                .then_some(binding.entity),
        )
    }

    /// Every member of `Family`, in role-name order, once the contract agrees
    /// with the family's.
    pub(in crate::domain_computation::primary_graph) fn family_members<Family>(
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
        use std::ops::Bound::{Excluded, Unbounded};
        use worth_query_declaration::facade::application_operation::WorthQueryApplicationDeclaredOutputRoleFamily;

        const { <Family as WorthQueryApplicationDeclaredOutputRoleFamily>::DECLARED };
        if self.contract_type != Some(TypeId::of::<Family::Contract>()) {
            return Err(WorthQueryApplicationOutputProjectionDenial::ForeignContract);
        }
        let prefix = Family::PREFIX;
        Ok(self
            .roles
            .range::<str, _>((Excluded(prefix), Unbounded))
            .take_while(move |(role, _)| role.starts_with(prefix))
            .map(|(role, binding)| {
                if binding.entity_type != TypeId::of::<Family::Entity>() {
                    return Err(WorthQueryApplicationOutputProjectionDenial::EntityMismatch);
                }
                Ok((role.as_str(), binding.posture, binding.entity))
            }))
    }
}
