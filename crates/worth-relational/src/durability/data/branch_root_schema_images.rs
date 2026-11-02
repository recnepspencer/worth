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
mod encoding;
mod registrations;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct DurableBranchRootSchemaImage {
    format_version: u16,
    carrier_digest: [u8; 32],
    schema_root: [u8; 32],
    descriptor_semantics_version: DescriptorSemanticsVersion,
    entity_registrations: Vec<DurableEntitySchemaRegistration>,
    relation_registrations: Vec<DurableRelationSchemaRegistration>,
    retained_aspect_contracts: Vec<PortableAspectContract>,
    // Appended optional tail preserves physical-only sequence and map encodings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    derived_relations: Vec<crate::derived_relation::DerivedRelationDefinitionParts>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    projected_member_domains: Vec<crate::projected_member_domain::ProjectedMemberDomainParts>,
    #[serde(default)]
    content_formats: crate::content::RelationalContentSchemaSnapshot,
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
            derived_relations: registry.derived_parts(),
            projected_member_domains: registry.projected_domain_parts(),
            content_formats: registry.content_formats.snapshot(),
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
        registry = registry
            .register_projected_member_domains(self.projected_member_domains.clone())
            .map_err(|error| format!("projected domain readmission denied: {error:?}"))?;
        if registry.projected_domain_parts() != self.projected_member_domains {
            return Err("branch-root projected domains are not canonical".to_owned());
        }
        let relations = self
            .relation_registrations
            .iter()
            .map(DurableRelationSchemaRegistration::readmit)
            .collect::<Result<Vec<_>, _>>()?;
        registry = crate::schema::source_catalog::install(
            registry,
            relations,
            self.derived_relations
                .iter()
                .cloned()
                .map(crate::schema::source_catalog::DerivedCatalogEntry::Meaning),
        )
        .map_err(|denial| format!("source catalog readmission denied: {denial:?}"))?;
        if registry.derived_parts() != self.derived_relations {
            return Err("branch-root derived definitions are not canonical".to_owned());
        }
        registry.content_formats = crate::content::RelationalContentFormatRegistry::from_recorded(
            self.content_formats.clone(),
        )
        .map_err(|cause| format!("content declarations refused: {cause:?}"))?;
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
        crate::derived_relation::canonical::update_digest(&mut digest, &self.derived_relations);
        crate::projected_member_domain::canonical::update_digest(
            &mut digest,
            &self.projected_member_domains,
        );
        if !self.content_formats.is_empty() {
            digest.update(b"worth.native.content-schema-carrier.v1\0");
            digest.update(rmp_serde::to_vec(&self.content_formats)?);
        }
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
