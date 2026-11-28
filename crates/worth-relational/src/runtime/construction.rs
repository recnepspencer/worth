use crate::runtime::{
    DurabilitySubsystem, HistorySubsystem, LineageSubsystem, PublicationSubsystem,
    RecordIdentitySubsystem, RuntimeServices, RuntimeSubsystem, SchemaContractRuntimeSubsystem,
    StorageSubsystem, VisibilitySubsystem,
};
use crate::validation::data::CustomInvariantRegistration;
use crate::validation::FrozenCustomInvariantRegistry;
use worth_relational_contracts::internal_port::symbols::data::string_interner::StringInternerOwnerPort as _;
use worth_relational_contracts::internal_port::validation::data::rules::InvariantRuleOwnerPort as _;

use super::state::{
    RelationalRuntimeConfiguration, RelationalRuntimeOwner, RelationalRuntimePublicationOwner,
};
use super::{RelationalRuntime, RelationalRuntimeConstructionDenial as Denial};

use super::fork_denial::RelationalRuntimeForkDenial;

#[derive(Debug, Default)]
pub(crate) struct RuntimeExtensions {
    custom_invariants: Vec<CustomInvariantRegistration>,
    managed_sources: Vec<crate::validation::candidate_source::ManagedIndexSourceProgram>,
    durable_medium: Option<Box<dyn crate::durability::published_medium::RelationalDurableMedium>>,
}

impl RuntimeExtensions {
    pub(crate) fn new(custom_invariants: Vec<CustomInvariantRegistration>) -> Self {
        Self {
            custom_invariants,
            managed_sources: Vec::new(),
            durable_medium: None,
        }
    }

    pub(crate) fn with_managed_index_sources(
        mut self,
        programs: Vec<crate::validation::candidate_source::ManagedIndexSourceProgram>,
    ) -> Self {
        self.managed_sources = programs;
        self
    }

    pub(in crate::runtime) fn is_home_configuration_only(&self) -> bool {
        self.custom_invariants.is_empty()
            && self.managed_sources.is_empty()
            && self.durable_medium.is_none()
    }

    pub(crate) fn with_durable_medium(
        mut self,
        durable_medium: Option<
            Box<dyn crate::durability::published_medium::RelationalDurableMedium>,
        >,
    ) -> Self {
        self.durable_medium = durable_medium;
        self
    }

    fn build_schema_contract_runtime_subsystem(
        &self,
        config: &super::RelationalRuntimeConfig,
    ) -> Result<SchemaContractRuntimeSubsystem, Denial> {
        let mut schema_contract_runtime =
            <SchemaContractRuntimeSubsystem as RuntimeSubsystem>::new(config);
        schema_contract_runtime.custom_invariant_registries =
            FrozenCustomInvariantRegistry::compile(
                &config.schema.registry,
                self.custom_invariants.clone(),
                self.managed_sources.clone(),
                crate::schema::SchemaCompilationControl::Configuration,
            )
            .map_err(|cause| match cause {
                crate::validation::CustomInvariantCompilationDenial::Source(cause) => {
                    Denial::ManagedIndexSource(cause)
                }
                crate::validation::CustomInvariantCompilationDenial::Duplicate(cause) => {
                    Denial::CustomInvariant(cause)
                }
                crate::validation::CustomInvariantCompilationDenial::Execution(cause) => {
                    Denial::Execution(cause)
                }
                crate::validation::CustomInvariantCompilationDenial::CapacityOverflow => {
                    Denial::Execution(
                        crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
                    )
                }
            })?;
        Ok(schema_contract_runtime)
    }

    fn build_runtime_services(&self) -> RuntimeServices {
        <RuntimeServices as RuntimeSubsystem>::new(&())
    }
}

impl RelationalRuntime {
    pub fn new(config: super::RelationalRuntimeConfig) -> Result<Self, Denial> {
        Self::build_from_extensions(config, RuntimeExtensions::default())
    }

    pub(crate) fn build_from_extensions(
        config: super::RelationalRuntimeConfig,
        mut extensions: RuntimeExtensions,
    ) -> Result<Self, Denial> {
        for (registration, declaration) in config
            .schema
            .invariant_catalog
            .registrations
            .iter()
            .enumerate()
        {
            if !declaration
                .rule
                .supports_execution_point(declaration.execution_point)
            {
                return Err(Denial::InvariantExecutionPoint {
                    registration,
                    execution_point: declaration.execution_point,
                });
            }
        }
        let schema_contract_runtime =
            extensions.build_schema_contract_runtime_subsystem(&config)?;
        let mut symbols = crate::symbols::data::StringInterner::default();
        symbols
            .import_canonical_snapshot(&config.identity.symbol_table)
            .map_err(Denial::SymbolTable)?;
        // Validate against the widest possible runtime prefix before issuing an
        // identity. Every future rebind then has no longer a decimal prefix.
        crate::branch::relational_branch_observation(
            u64::MAX,
            &config.history.main_branch.0,
            worth_foundational::FoundationalBranchTarget::empty(),
            worth_foundational::FoundationalBranchReferenceGeneration::initial(),
        )
        .map_err(Denial::MainBranch)?;
        let main_cell = crate::branch::RelationalBranchReferenceCell::empty(
            0,
            config.history.main_branch.clone(),
        )
        .map_err(Denial::MainBranch)?;
        let services = extensions.build_runtime_services();
        services.symbols.replace(symbols);
        let mut history = crate::runtime::state::subsystems::history::history_canonical_routes::RelationalPublicationHistory::from_branch(
            HistorySubsystem::from_main_cell(main_cell),
        );
        history
            .set_runtime_instance_id(services.runtime_instance_id())
            .map_err(Denial::MainBranch)?;
        let initial_root = crate::branch::RelationalBranchRoot::empty_with_schema(
            &config.schema.registry,
            crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
        );
        crate::runtime::state::subsystems::history::history_canonical_routes::retain_original_empty_birth(
            &history,
            initial_root.retained_schema_authority(),
        );
        // The new registry contains exactly one fresh cell, bound to this
        // runtime, with a nonzero retention budget and no escaped handles.
        history
            .install_branch_root(&config.history.main_branch, initial_root)
            .expect(/* Proof: The fresh registry has one same-runtime cell, a nonzero retention budget and no escaped handles; its first root installation cannot exceed retention capacity. */ "the configured main branch fits the initial retention budget");
        let mut durability = <DurabilitySubsystem as RuntimeSubsystem>::new(&config);
        if let Some(medium) = extensions.durable_medium.take() {
            durability.attach_medium(medium);
        }
        let partitions = <StorageSubsystem as RuntimeSubsystem>::new(&());
        let working_set = partitions.working_set(config.storage.residency_budget);
        durability
            .checkpoint_selection()
            .set_working_set(working_set);
        let lineage = LineageSubsystem::new();
        durability
            .checkpoint_selection()
            .register_naming_ack_hook(lineage.naming_ack_hook());
        let visibility = VisibilitySubsystem::new(&config);
        Ok(Self::from_state(crate::runtime::RelationalRuntimeState {
            history,
            indexes: Default::default(),
            lineage,
            durability,
            record_identity: <RecordIdentitySubsystem as RuntimeSubsystem>::new(&()),
            services,
            owner_lifecycle: RelationalRuntimeOwner::new(),
            publication_owner: RelationalRuntimePublicationOwner::standalone(),
            partitions,
            visibility,
            publication: PublicationSubsystem::default(),
            configuration: RelationalRuntimeConfiguration::new(config, schema_contract_runtime),
        }))
    }

    pub fn fork(&self) -> Result<Self, RelationalRuntimeForkDenial> {
        let binding = self.configuration_binding();
        let configuration = binding.operation();
        let mut history = self.history.fork_snapshot()?;
        let services = RuntimeSubsystem::fork(&self.services);
        let runtime_instance_id = services.runtime_instance_id();
        history
            .bind_fork_runtime(runtime_instance_id)
            .map_err(RelationalRuntimeForkDenial::BranchIdentityRebind)?;
        let config = configuration.config.as_ref().clone();
        let schema_contract_runtime =
            RuntimeSubsystem::fork(configuration.schema_contract_runtime.as_ref());
        let (lineage, pinned_naming) = self.lineage.fork_with_pinned_cold();
        let durability = RuntimeSubsystem::fork(&self.durability);
        if let Some(custody) = pinned_naming.and_then(|pinned| pinned.custody()) {
            durability
                .checkpoint_selection()
                .pin_detached_naming_directory(custody);
        }
        durability
            .checkpoint_selection()
            .register_naming_ack_hook(lineage.naming_ack_hook());
        Ok(Self::from_state(crate::runtime::RelationalRuntimeState {
            configuration: RelationalRuntimeConfiguration::new(config, schema_contract_runtime),
            partitions: self.fork_partitions(),
            visibility: self.visibility.fork(),
            publication: self.publication.fork(),
            history,
            indexes: self.indexes.fork(),
            lineage,
            durability,
            record_identity: RuntimeSubsystem::fork(&self.record_identity),
            services,
            owner_lifecycle: RelationalRuntimeOwner::new(),
            publication_owner: RelationalRuntimePublicationOwner::fork_from(
                &self.publication_owner,
            ),
        }))
    }
}
