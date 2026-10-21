use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use worth_foundational::facade::{AspectBinding, AspectContract, PortableAspectContract};

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;
use crate::merge::data::{AspectMergePolicyDeclaration, IdentityBasisDeclaration};
use crate::schema::data::{
    DeclaredAspectContractBinding, DescriptorSemanticsVersion, EntityKindRegistration,
    KindAspectContractDeclarations, RelationIntegrityDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

const ROOT_SCHEMA_IMAGE_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurableBranchRootSchemaImage {
    format_version: u16,
    carrier_digest: [u8; 32],
    schema_root: [u8; 32],
    descriptor_semantics_version: DescriptorSemanticsVersion,
    entity_registrations: Vec<DurableEntitySchemaRegistration>,
    relation_registrations: Vec<DurableRelationSchemaRegistration>,
    retained_aspect_contracts: Vec<PortableAspectContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurableEntitySchemaRegistration {
    kind_id: KindId,
    kind_name: String,
    schema_id: SchemaId,
    schema_version_id: SchemaVersionId,
    declarations: DurableKindAspectDeclarations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DurableRelationSchemaRegistration {
    kind_id: KindId,
    kind_name: String,
    schema_id: SchemaId,
    schema_version_id: SchemaVersionId,
    cross_context_policy: CrossContextPolicy,
    cascade_delete_policy: CascadeDeletePolicy,
    declarations: DurableKindAspectDeclarations,
    relation_integrity: RelationIntegrityDeclarations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DurableKindAspectDeclarations {
    aspects: Vec<DurableDeclaredAspectBinding>,
    identity_declarations: Vec<IdentityBasisDeclaration>,
    merge_policy_declarations: Vec<AspectMergePolicyDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DurableDeclaredAspectBinding {
    binding: AspectBinding,
    contract: PortableAspectContract,
}

impl DurableBranchRootSchemaImage {
    pub(crate) fn capture(
        authority: &crate::branch::RelationalBranchRootSchemaAuthority,
    ) -> Result<Self, rmp_serde::encode::Error> {
        let registry = authority.registry();
        let entity_registrations = registry
            .entity_kinds
            .values()
            .map(DurableEntitySchemaRegistration::capture)
            .collect();
        let relation_registrations = registry
            .relation_kinds
            .values()
            .map(DurableRelationSchemaRegistration::capture)
            .collect();
        let mut image = Self {
            format_version: ROOT_SCHEMA_IMAGE_VERSION,
            carrier_digest: [0; 32],
            schema_root: registry.authority_digest_bytes(),
            descriptor_semantics_version: authority.descriptor_semantics_version(),
            entity_registrations,
            relation_registrations,
            retained_aspect_contracts: authority
                .retained_aspect_contracts()
                .iter()
                .map(PortableAspectContract::from_contract)
                .collect(),
        };
        image.carrier_digest = image.recompute_digest()?;
        Ok(image)
    }

    pub(crate) const fn carrier_digest(&self) -> [u8; 32] {
        self.carrier_digest
    }

    pub(crate) const fn schema_root(&self) -> [u8; 32] {
        self.schema_root
    }

    pub(crate) const fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion {
        self.descriptor_semantics_version
    }

    #[cfg(test)]
    pub(crate) fn corrupt_schema_root_for_test(&mut self) {
        self.schema_root[0] ^= 0x80;
    }

    pub(crate) fn readmit_registry(
        &self,
    ) -> Result<(RelationalSchemaRegistry, Vec<AspectContract>), String> {
        if self.format_version != ROOT_SCHEMA_IMAGE_VERSION {
            return Err(format!(
                "unsupported branch-root schema image version `{}`",
                self.format_version
            ));
        }
        let digest = self
            .recompute_digest()
            .map_err(|error| format!("schema carrier encoding failed: {error}"))?;
        if digest != self.carrier_digest {
            return Err("branch-root schema carrier digest mismatch".to_owned());
        }
        require_canonical_kind_order(
            self.entity_registrations.iter().map(|entry| entry.kind_id),
            "entity",
        )?;
        require_canonical_contract_order(&self.retained_aspect_contracts)?;
        require_canonical_kind_order(
            self.relation_registrations
                .iter()
                .map(|entry| entry.kind_id),
            "relation",
        )?;
        let mut registry = RelationalSchemaRegistry::new();
        for registration in &self.entity_registrations {
            registry = registry
                .register_entity_kind(registration.readmit()?)
                .map_err(|error| format!("entity schema readmission denied: {error:?}"))?;
        }
        for registration in &self.relation_registrations {
            registry = registry
                .register_relation_kind(registration.readmit()?)
                .map_err(|error| format!("relation schema readmission denied: {error:?}"))?;
        }
        if registry.authority_digest_bytes() != self.schema_root {
            return Err("branch-root schema authority digest mismatch".to_owned());
        }
        let retained_aspect_contracts = self
            .retained_aspect_contracts
            .iter()
            .map(|contract| {
                contract.readmit().map_err(|denial| {
                    format!("retained aspect contract readmission denied: {denial:?}")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((registry, retained_aspect_contracts))
    }

    fn recompute_digest(&self) -> Result<[u8; 32], rmp_serde::encode::Error> {
        let encoded = rmp_serde::to_vec(&(
            self.format_version,
            self.schema_root,
            self.descriptor_semantics_version,
            &self.entity_registrations,
            &self.relation_registrations,
            &self.retained_aspect_contracts,
        ))?;
        let mut digest = Sha256::new();
        digest.update(b"worth.relational.branch-root-schema-image.v1\0");
        digest.update(encoded);
        Ok(digest.finalize().into())
    }
}

fn require_canonical_contract_order(contracts: &[PortableAspectContract]) -> Result<(), String> {
    let mut previous = None;
    for contract in contracts {
        let basis = (contract.key(), contract.identity(), contract.revision());
        if previous.is_some_and(|previous| previous >= basis) {
            return Err("branch-root retained aspect contracts are not canonical".to_owned());
        }
        previous = Some(basis);
    }
    Ok(())
}

impl DurableEntitySchemaRegistration {
    pub(crate) fn capture_bytes(registration: &EntityKindRegistration) -> Option<usize> {
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
            if let crate::merge::data::AspectMergePolicyKind::Custom(identity) = &policy.policy {
                bytes = bytes.checked_add(identity.name.len())?;
            }
        }
        for identity in &declarations.identity_declarations {
            if let crate::merge::data::IdentityBasisScope::AspectKey(key) = &identity.scope {
                bytes = bytes.checked_add(key.owned_allocation_capacity_bytes())?;
            }
            match &identity.basis {
                crate::merge::data::IdentityBasisKind::DeclaredKeySet(keys) => {
                    bytes = bytes.checked_add(
                        keys.len().checked_mul(std::mem::size_of::<
                            worth_foundational::facade::AspectKey,
                        >())?,
                    )?;
                    for key in keys.iter() {
                        bytes = bytes.checked_add(key.owned_allocation_capacity_bytes())?;
                    }
                }
                crate::merge::data::IdentityBasisKind::Custom(identity) => {
                    bytes = bytes.checked_add(identity.name.len())?;
                }
                _ => {}
            }
        }
        bytes.checked_mul(4)
    }
    pub(crate) fn capture(registration: &EntityKindRegistration) -> Self {
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

    pub(crate) fn readmit(&self) -> Result<EntityKindRegistration, String> {
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
    fn capture(registration: &RelationKindRegistration) -> Self {
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

    fn readmit(&self) -> Result<RelationKindRegistration, String> {
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

fn require_canonical_kind_order(
    kind_ids: impl Iterator<Item = KindId>,
    domain: &str,
) -> Result<(), String> {
    let mut previous = None;
    for kind_id in kind_ids {
        if previous.is_some_and(|previous| previous >= kind_id) {
            return Err(format!(
                "branch-root {domain} schema registrations are not canonical"
            ));
        }
        previous = Some(kind_id);
    }
    Ok(())
}
