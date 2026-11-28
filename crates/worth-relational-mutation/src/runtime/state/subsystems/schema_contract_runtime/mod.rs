use crate::contracts::schema::compilation_control::SchemaCompilationControl;
use crate::validation::custom_registry::FrozenCustomInvariantRegistry;
pub mod native_points;
pub mod native_registrations;
use crate::contracts::identity::data::primitives::KindId;
use native_registrations::NativeRelationIntegrityInventory;
use std::collections::BTreeSet;

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
pub struct SchemaContractRuntimeSubsystem {
    pub(crate) invariant_points: std::sync::Arc<native_points::NativeInvariantPoints>,
    pub schema_authority_digest: [u8; 32],
    pub aspect_contract_plans: worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalAspectPlans,
    pub relation_integrity_registrations: std::sync::Arc<NativeRelationIntegrityInventory>,
    pub custom_invariant_registries: FrozenCustomInvariantRegistry,
    pub custom_invariant_generation: u64,
    pub initial_custom_invariants_sealed: bool,
    pub home_readmission_metadata_kinds: std::sync::Arc<HomeReadmissionMetadataKinds>,
    pub home_configuration_backing: Option<std::sync::Arc<HomeConfigurationBacking>>,
}

/// Funding retained with the exact compiled receiver inventory.
#[derive(Debug)]
pub struct HomeConfigurationBacking {
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl SchemaContractRuntimeSubsystem {
    /// Initial installation and a Home target lower the same complete inventory.
    /// The supplied aspect plans are the exact selected root schema's plans.
    pub fn for_schema(
        registry: &crate::contracts::schema::data::RelationalSchemaRegistry,
        declared: &[crate::contracts::validation::data::catalog::InvariantRegistration],
        aspect_contract_plans: worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalAspectPlans,
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
    pub fn for_schema_compilation(
        registry: &crate::contracts::schema::data::RelationalSchemaRegistry,
        declared: &[crate::contracts::validation::data::catalog::InvariantRegistration],
        aspect_contract_plans: worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalAspectPlans,
        custom_invariant_registries: FrozenCustomInvariantRegistry,
        custom_invariant_generation: u64,
        initial_custom_invariants_sealed: bool,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>
    {
        let registrations = NativeRelationIntegrityInventory::compile(registry, control)?;
        let aspect_exclusion = aspect_contract_plans.entity_plans.values().try_fold(
            false,
            |installed, plan| {
                control.checkpoint()?;
                Ok::<_, crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>(
                    installed || !plan.exclusion_groups.is_empty(),
                )
            },
        )?;
        let invariant_points = native_points::NativeInvariantPoints::compile(
            declared,
            &registrations,
            aspect_exclusion,
            control,
        )?;
        Ok(Self {
            invariant_points,
            schema_authority_digest: match control {
                SchemaCompilationControl::Configuration => registry.authority_digest_bytes(),
                SchemaCompilationControl::Request(request) => {
                    worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority::registry_digest_for_request(
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

    pub fn with_home_readmission_metadata_kinds(
        mut self,
        kinds: std::sync::Arc<HomeReadmissionMetadataKinds>,
    ) -> Self {
        self.home_readmission_metadata_kinds = kinds;
        self
    }
}
