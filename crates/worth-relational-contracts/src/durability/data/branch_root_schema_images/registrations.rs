//! Portable kind-registration capture and structural readmission.
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;
use crate::schema::data::aspect_merge_policy::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind,
};
use crate::schema::data::identity_declarations::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
};
use crate::schema::data::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationIntegrityDeclarations, RelationKindRegistration, SchemaId, SchemaVersionId,
};
use serde::{Deserialize, Serialize};
use worth_foundational::facade::{AspectBinding, PortableAspectContract};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableEntitySchemaRegistration {
    pub kind_id: KindId,
    kind_name: String,
    schema_id: SchemaId,
    schema_version_id: SchemaVersionId,
    pub declarations: DurableKindAspectDeclarations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableRelationSchemaRegistration {
    pub kind_id: KindId,
    kind_name: String,
    schema_id: SchemaId,
    schema_version_id: SchemaVersionId,
    cross_context_policy: CrossContextPolicy,
    cascade_delete_policy: CascadeDeletePolicy,
    pub declarations: DurableKindAspectDeclarations,
    relation_integrity: RelationIntegrityDeclarations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableKindAspectDeclarations {
    aspects: Vec<DurableDeclaredAspectBinding>,
    pub identity_declarations: Vec<IdentityBasisDeclaration>,
    pub merge_policy_declarations: Vec<AspectMergePolicyDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableDeclaredAspectBinding {
    binding: AspectBinding,
    contract: PortableAspectContract,
}

impl DurableEntitySchemaRegistration {
    /// Capacities, not entry counts: the decoder supplies its checked Vec
    /// growth/overlap bounds. Nested strings/contracts remain separately quoted.
    pub fn declaration_roster_storage_bytes(
        aspect_capacity: usize,
        identity_capacity: usize,
        policy_capacity: usize,
    ) -> Option<usize> {
        std::alloc::Layout::array::<DurableDeclaredAspectBinding>(aspect_capacity)
            .ok()?
            .size()
            .checked_add(
                std::alloc::Layout::array::<IdentityBasisDeclaration>(identity_capacity)
                    .ok()?
                    .size(),
            )?
            .checked_add(
                std::alloc::Layout::array::<AspectMergePolicyDeclaration>(policy_capacity)
                    .ok()?
                    .size(),
            )
    }
}

impl DurableEntitySchemaRegistration {
    pub fn capture_bytes(registration: &EntityKindRegistration) -> Option<usize> {
        let declarations = &registration.aspect_contract_declarations;
        let mut bytes = std::mem::size_of::<Self>()
            .checked_add(registration.kind_name.len())?
            .checked_add(registration.schema_id.0.len())?
            .checked_add(
                declarations
                    .aspects
                    .len()
                    .checked_mul(std::mem::size_of::<DurableDeclaredAspectBinding>())?,
            )?
            .checked_add(
                declarations
                    .identity_declarations
                    .len()
                    .checked_mul(std::mem::size_of::<IdentityBasisDeclaration>())?,
            )?
            .checked_add(
                declarations
                    .merge_policy_declarations
                    .len()
                    .checked_mul(std::mem::size_of::<AspectMergePolicyDeclaration>())?,
            )?;
        for aspect in &declarations.aspects {
            bytes = bytes
                .checked_add(aspect.binding.owned_allocation_capacity_bytes())?
                .checked_add(aspect.contract.owned_allocation_capacity_bytes())?;
        }
        for policy in &declarations.merge_policy_declarations {
            bytes = bytes.checked_add(policy.aspect_key.owned_allocation_capacity_bytes())?;
            if let AspectMergePolicyKind::Custom(identity) = &policy.policy {
                bytes = bytes.checked_add(identity.name.len())?;
            }
        }
        for identity in &declarations.identity_declarations {
            if let IdentityBasisScope::AspectKey(key) = &identity.scope {
                bytes = bytes.checked_add(key.owned_allocation_capacity_bytes())?;
            }
            match &identity.basis {
                IdentityBasisKind::DeclaredKeySet(keys) => {
                    bytes = bytes.checked_add(
                        keys.len().checked_mul(std::mem::size_of::<
                            worth_foundational::facade::AspectKey,
                        >())?,
                    )?;
                    for key in keys.iter() {
                        bytes = bytes.checked_add(key.owned_allocation_capacity_bytes())?;
                    }
                }
                IdentityBasisKind::Custom(identity) => {
                    bytes = bytes.checked_add(identity.name.len())?;
                }
                _ => {}
            }
        }
        bytes.checked_mul(4)
    }
    pub fn capture(registration: &EntityKindRegistration) -> Self {
        Self {
            kind_id: registration.kind_id,
            kind_name: registration.kind_name.clone(),
            schema_id: registration.schema_id.clone(),
            schema_version_id: registration.schema_version_id,
            declarations: DurableKindAspectDeclarations::capture(
                &registration.aspect_contract_declarations,
            ),
        }
    }

    pub fn readmit(&self) -> Result<EntityKindRegistration, String> {
        Ok(EntityKindRegistration {
            kind_id: self.kind_id,
            kind_name: self.kind_name.clone(),
            schema_id: self.schema_id.clone(),
            schema_version_id: self.schema_version_id,
            aspect_contract_declarations: self.declarations.readmit()?,
        })
    }
}

impl DurableRelationSchemaRegistration {
    pub fn capture(registration: &RelationKindRegistration) -> Self {
        Self {
            kind_id: registration.kind_id,
            kind_name: registration.kind_name.clone(),
            schema_id: registration.schema_id.clone(),
            schema_version_id: registration.schema_version_id,
            cross_context_policy: registration.cross_context_policy,
            cascade_delete_policy: registration.cascade_delete_policy,
            declarations: DurableKindAspectDeclarations::capture(
                &registration.aspect_contract_declarations,
            ),
            relation_integrity: registration.relation_integrity.clone(),
        }
    }

    pub fn readmit(&self) -> Result<RelationKindRegistration, String> {
        Ok(RelationKindRegistration {
            kind_id: self.kind_id,
            kind_name: self.kind_name.clone(),
            schema_id: self.schema_id.clone(),
            schema_version_id: self.schema_version_id,
            cross_context_policy: self.cross_context_policy,
            cascade_delete_policy: self.cascade_delete_policy,
            aspect_contract_declarations: self.declarations.readmit()?,
            relation_integrity: self.relation_integrity.clone(),
        })
    }
}

impl DurableKindAspectDeclarations {
    fn capture(declarations: &KindAspectContractDeclarations) -> Self {
        Self {
            aspects: declarations
                .aspects
                .iter()
                .map(|declaration| DurableDeclaredAspectBinding {
                    binding: declaration.binding.clone(),
                    contract: PortableAspectContract::from_contract(&declaration.contract),
                })
                .collect(),
            identity_declarations: declarations.identity_declarations.clone(),
            merge_policy_declarations: declarations.merge_policy_declarations.clone(),
        }
    }

    fn readmit(&self) -> Result<KindAspectContractDeclarations, String> {
        let aspects = self
            .aspects
            .iter()
            .map(|declaration| {
                declaration
                    .contract
                    .readmit()
                    .map(|contract| DeclaredAspectContractBinding {
                        binding: declaration.binding.clone(),
                        contract,
                    })
                    .map_err(|denial| format!("aspect contract readmission denied: {denial:?}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(KindAspectContractDeclarations::new(aspects)
            .with_identity_declarations(self.identity_declarations.clone())
            .with_merge_policy_declarations(self.merge_policy_declarations.clone()))
    }
}
