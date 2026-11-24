use crate::contracts::schema::compilation_control::SchemaCompilationControl;
use crate::validation::custom_registry::FrozenCustomInvariantRegistry;
pub mod native_points;
pub mod native_registrations;
use native_registrations::NativeRelationIntegrityInventory;

#[derive(Debug, Clone)]
pub struct SchemaContractRuntimeSubsystem {
    pub(crate) invariant_points: std::sync::Arc<native_points::NativeInvariantPoints>,
    pub schema_authority_digest: [u8; 32],
    pub aspect_contract_plans: worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalAspectPlans,
    pub relation_integrity_registrations: std::sync::Arc<NativeRelationIntegrityInventory>,
    pub custom_invariant_registries: FrozenCustomInvariantRegistry,
    pub custom_invariant_generation: u64,
    pub initial_custom_invariants_sealed: bool,
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
        let invariant_points =
            native_points::NativeInvariantPoints::compile(declared, &registrations, control)?;
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
            home_configuration_backing: None,
        })
    }
}
