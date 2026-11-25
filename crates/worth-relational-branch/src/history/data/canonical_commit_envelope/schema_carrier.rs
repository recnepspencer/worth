use serde::Deserialize;
use worth_foundational::facade::{AspectContract, PortableAspectContract};
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

use crate::identity::data::KindId;
use crate::merge::data::{
    identity::IdentityBasisDeclaration, policy::AspectMergePolicyDeclaration,
};
use crate::schema::data::{
    DescriptorSemanticsVersion, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationalSchemaRegistry,
};

const ROOT_SCHEMA_IMAGE_VERSION: u16 = 1;
pub mod allocation;
pub use allocation::DurableSchemaDeclarationMemory;
pub mod capture_memory;
pub mod digest;
pub mod encoding;
pub mod owner;
pub use worth_relational_contracts::internal_port::history::data::canonical_commit_envelope::schema_carrier::registrations;
pub use registrations::DurableEntitySchemaRegistration;
use registrations::{
    DurableDeclaredAspectBinding, DurableKindAspectDeclarations, DurableRelationSchemaRegistration,
};

#[derive(Debug, Clone)]
pub struct RecordedBranchSchemaImage(std::sync::Arc<SchemaImageData>);

/// Borrow the descriptor and declarations already admitted for one branch schema.
#[derive(Clone, Copy)]
pub struct SchemaImageSource<'a> {
    pub registry: &'a RelationalSchemaRegistry,
    pub retained_aspect_contracts: &'a [AspectContract],
    pub schema_root: [u8; 32],
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
}

#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct SchemaImageData {
    pub format_version: u16,
    carrier_digest: [u8; 32],
    pub schema_root: [u8; 32],
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
    pub entity_registrations: Vec<DurableEntitySchemaRegistration>,
    pub relation_registrations: Vec<DurableRelationSchemaRegistration>,
    pub retained_aspect_contracts: Vec<PortableAspectContract>,
    // Older optional declarations retain their decoding defaults. Their slots
    // are always emitted before the required content-format authority.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    derived_relations: Vec<crate::derived_relation::definition::DerivedRelationDefinitionParts>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    projected_member_domains: Vec<crate::projected_member_domain::ProjectedMemberDomainParts>,
    #[serde(skip)]
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    content_formats: crate::content::RelationalContentSchemaSnapshot,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    persistent_name_schemes: Vec<crate::schema::data::SchemaAuthorityNamingSchemeSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    kind_name_schemes: Vec<crate::schema::data::SchemaAuthorityKindNamingBindingSnapshot>,
}

impl RecordedBranchSchemaImage {
    pub fn carrier_digest(&self) -> [u8; 32] {
        self.0.carrier_digest
    }

    pub fn schema_root(&self) -> [u8; 32] {
        self.0.schema_root
    }

    pub fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion {
        self.0.descriptor_semantics_version
    }

    pub(crate) fn corrupt_schema_root(&mut self) {
        let bytes = rmp_serde::to_vec(self).expect("valid test schema image");
        *self = rmp_serde::from_slice(&bytes).expect("independent test schema image");
        std::sync::Arc::get_mut(&mut self.0)
            .expect("unique test image")
            .schema_root[0] ^= 0x80;
    }

    pub fn readmit_registry(
        &self,
    ) -> Result<(RelationalSchemaRegistry, Vec<AspectContract>), String> {
        if self.0.format_version != ROOT_SCHEMA_IMAGE_VERSION {
            return Err(format!(
                "unsupported branch-root schema image version `{}`",
                self.0.format_version
            ));
        }
        let digest = self
            .0
            .recompute_digest()
            .map_err(|error| format!("schema carrier encoding failed: {error}"))?;
        if digest != self.0.carrier_digest {
            return Err("branch-root schema carrier digest mismatch".to_owned());
        }
        require_canonical_kind_order(
            self.0
                .entity_registrations
                .iter()
                .map(|entry| entry.kind_id),
            "entity",
        )?;
        require_canonical_contract_order(&self.0.retained_aspect_contracts)?;
        require_canonical_kind_order(
            self.0
                .relation_registrations
                .iter()
                .map(|entry| entry.kind_id),
            "relation",
        )?;
        let mut registry = RelationalSchemaRegistry::new();
        for registration in &self.0.entity_registrations {
            registry = registry
                .register_entity_kind(registration.readmit()?)
                .map_err(|error| format!("entity schema readmission denied: {error:?}"))?;
        }
        registry = registry
            .register_projected_member_domains(self.0.projected_member_domains.clone())
            .map_err(|error| format!("projected domain readmission denied: {error:?}"))?;
        if registry.projected_domain_parts() != self.0.projected_member_domains {
            return Err("branch-root projected domains are not canonical".to_owned());
        }
        let relations = self
            .0
            .relation_registrations
            .iter()
            .map(DurableRelationSchemaRegistration::readmit)
            .collect::<Result<Vec<_>, _>>()?;
        registry = crate::schema::source_catalog::install(
            registry,
            relations,
            self.0
                .derived_relations
                .iter()
                .cloned()
                .map(crate::schema::source_catalog::DerivedCatalogEntry::Meaning),
        )
        .map_err(|denial| format!("source catalog readmission denied: {denial:?}"))?;
        if registry.derived_parts() != self.0.derived_relations {
            return Err("branch-root derived definitions are not canonical".to_owned());
        }
        (*registry.owner_content_formats_mut()) =
            crate::content::RelationalContentFormatRegistry::from_recorded(
                self.0.content_formats.clone(),
            )
            .map_err(|cause| format!("content declarations refused: {cause:?}"))?;
        registry = registry.readmit_persistent_name_schemes(
            &self.0.persistent_name_schemes,
            &self.0.kind_name_schemes,
        )?;
        if registry.authority_digest_bytes() != self.0.schema_root {
            return Err("branch-root schema authority digest mismatch".to_owned());
        }
        let retained_aspect_contracts = self
            .0
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

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait RecordedBranchSchemaImageFixturePort: Sized {
    fn corrupt_schema_root_for_test(&mut self);
}

impl RecordedBranchSchemaImageFixturePort for RecordedBranchSchemaImage {
    fn corrupt_schema_root_for_test(&mut self) {
        self.corrupt_schema_root()
    }
}

impl RecordedBranchSchemaImage {
    #[doc(hidden)]
    pub fn schema_data_for_test(&self) -> &std::sync::Arc<SchemaImageData> {
        &self.0
    }
}
