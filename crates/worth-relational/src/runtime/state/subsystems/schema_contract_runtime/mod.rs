use crate::identity::data::KindId;
use crate::runtime::state::subsystems::RuntimeSubsystem;
use crate::schema::{lower_aspect_plans, SchemaCompilationControl};
use crate::validation::FrozenCustomInvariantRegistry;
use std::collections::BTreeSet;
mod native_points;
mod native_registrations;
use native_registrations::NativeRelationIntegrityInventory;

/// Owner-admitted internal metadata kinds that Home readmission may scan on a
/// recovered branch even while generated records are suspended.
///
/// Sealed during initial schema installation and never selectable by serving
/// callers or post-recovery observers.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct HomeReadmissionMetadataKinds {
    entity_kinds: BTreeSet<KindId>,
    relation_kinds: BTreeSet<KindId>,
}

impl HomeReadmissionMetadataKinds {
    pub fn new(
        entity_kinds: impl IntoIterator<Item = KindId>,
        relation_kinds: impl IntoIterator<Item = KindId>,
    ) -> Self {
        Self {
            entity_kinds: entity_kinds.into_iter().collect(),
            relation_kinds: relation_kinds.into_iter().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entity_kinds.is_empty() && self.relation_kinds.is_empty()
    }

    pub fn contains_entity_kind(&self, kind_id: KindId) -> bool {
        self.entity_kinds.contains(&kind_id)
    }

    pub fn contains_relation_kind(&self, kind_id: KindId) -> bool {
        self.relation_kinds.contains(&kind_id)
    }

    pub fn entity_kinds(&self) -> &BTreeSet<KindId> {
        &self.entity_kinds
    }

    pub fn relation_kinds(&self) -> &BTreeSet<KindId> {
        &self.relation_kinds
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SchemaContractRuntimeSubsystem {
    pub(crate) invariant_points: std::sync::Arc<native_points::NativeInvariantPoints>,
    pub(crate) schema_authority_digest: [u8; 32],
    pub(crate) aspect_contract_plans: crate::branch::RetainedRelationalAspectPlans,
    pub(crate) relation_integrity_registrations: std::sync::Arc<NativeRelationIntegrityInventory>,
    pub(crate) custom_invariant_registries: FrozenCustomInvariantRegistry,
    pub(crate) custom_invariant_generation: u64,
    pub(crate) initial_custom_invariants_sealed: bool,
    pub(crate) home_readmission_metadata_kinds: std::sync::Arc<HomeReadmissionMetadataKinds>,
    pub(in crate::runtime) home_configuration_backing: Option<
        std::sync::Arc<
            crate::runtime::home_program_transition::configuration::HomeConfigurationBacking,
        >,
    >,
}

impl RuntimeSubsystem for SchemaContractRuntimeSubsystem {
    type Config = crate::config::data::RelationalRuntimeConfig;

    fn new(config: &Self::Config) -> Self {
        Self::for_schema(
            &config.schema.registry,
            &config.schema.invariant_catalog.registrations,
            crate::branch::RetainedRelationalAspectPlans::from_configuration(lower_aspect_plans(
                &config.schema.registry,
            )),
            FrozenCustomInvariantRegistry::default(),
            0,
            false,
        )
    }

    fn fork(&self) -> Self {
        self.clone()
    }
}

impl SchemaContractRuntimeSubsystem {
    /// Initial installation and a Home target lower the same complete inventory.
    /// The supplied aspect plans are the exact selected root schema's plans.
    pub(crate) fn for_schema(
        registry: &crate::schema::data::RelationalSchemaRegistry,
        declared: &[crate::validation::data::InvariantRegistration],
        aspect_contract_plans: crate::branch::RetainedRelationalAspectPlans,
        custom_invariant_registries: FrozenCustomInvariantRegistry,
        custom_invariant_generation: u64,
        initial_custom_invariants_sealed: bool,
    ) -> Self {
        Self::for_schema_compilation(
            registry,
            declared,
            aspect_contract_plans,
            custom_invariant_registries,
            custom_invariant_generation,
            initial_custom_invariants_sealed,
            SchemaCompilationControl::Configuration,
        )
        .expect("configuration native registration compilation has no request refusal")
    }
    pub(crate) fn for_schema_compilation(
        registry: &crate::schema::data::RelationalSchemaRegistry,
        declared: &[crate::validation::data::InvariantRegistration],
        aspect_contract_plans: crate::branch::RetainedRelationalAspectPlans,
        custom_invariant_registries: FrozenCustomInvariantRegistry,
        custom_invariant_generation: u64,
        initial_custom_invariants_sealed: bool,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        let registrations = NativeRelationIntegrityInventory::compile(registry, control)?;
        let invariant_points =
            native_points::NativeInvariantPoints::compile(declared, &registrations, control)?;
        Ok(Self {
            invariant_points,
            schema_authority_digest: match control {
                SchemaCompilationControl::Configuration => registry.authority_digest_bytes(),
                SchemaCompilationControl::Request(request) => {
                    crate::branch::RelationalBranchRootSchemaAuthority::registry_digest_for_request(
                        registry, &request,
                    )?
                }
            },
            aspect_contract_plans,
            relation_integrity_registrations: registrations,
            custom_invariant_registries,
            custom_invariant_generation,
            initial_custom_invariants_sealed,
            home_readmission_metadata_kinds: std::sync::Arc::new(
                HomeReadmissionMetadataKinds::default(),
            ),
            home_configuration_backing: None,
        })
    }

    pub(crate) fn with_home_readmission_metadata_kinds(
        mut self,
        kinds: std::sync::Arc<HomeReadmissionMetadataKinds>,
    ) -> Self {
        self.home_readmission_metadata_kinds = kinds;
        self
    }
}
