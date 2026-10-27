use crate::commit_strategies::{
    data::CommitStrategyExecutionRegistration, FrozenCommitStrategyExecutorRegistry,
    FrozenCommitStrategyRegistry,
};
use crate::runtime::{
    CommitStrategiesSubsystem, DurabilitySubsystem, HistorySubsystem, IndexingSubsystem,
    LineageSubsystem, PublicationSubsystem, RecordIdentitySubsystem, RuntimeServices,
    RuntimeSubsystem, SchemaContractRuntimeSubsystem, StorageSubsystem, VisibilitySubsystem,
};
use crate::validation::data::CustomInvariantRegistration;
use crate::validation::FrozenCustomInvariantRegistry;

use super::state::{
    RelationalRuntimeConfiguration, RelationalRuntimeOwner, RelationalRuntimePublicationOwner,
};
use super::{RelationalRuntime, RelationalRuntimeConstructionDenial as Denial};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRuntimeForkDenial {
    PublicationInFlight,
    PerformedPublicationRequiresSettlement {
        commit_id: crate::history::data::CommitId,
    },
    IdentityCapacityExhausted,
    CanonicalInventoryInvalid,
    BranchIdentityRebind(crate::branch::RelationalBranchObservationConstructionDenial),
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeExtensions {
    custom_invariants: Vec<CustomInvariantRegistration>,
    commit_strategy_executors: Vec<CommitStrategyExecutionRegistration>,
    post_commit_consumer: Option<std::sync::Arc<dyn crate::publication::PostCommitConsumer>>,
    durable_medium: Option<Box<dyn crate::durability::medium::RelationalDurableMedium>>,
}

impl RuntimeExtensions {
    pub(crate) fn new(
        custom_invariants: Vec<CustomInvariantRegistration>,
        commit_strategy_executors: Vec<CommitStrategyExecutionRegistration>,
    ) -> Self {
        Self {
            custom_invariants,
            commit_strategy_executors,
            post_commit_consumer: None,
            durable_medium: None,
        }
    }

    pub(crate) fn with_post_commit_consumer(
        mut self,
        post_commit_consumer: std::sync::Arc<dyn crate::publication::PostCommitConsumer>,
    ) -> Self {
        self.post_commit_consumer = Some(post_commit_consumer);
        self
    }

    pub(crate) fn with_durable_medium(
        mut self,
        durable_medium: Option<Box<dyn crate::durability::medium::RelationalDurableMedium>>,
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
            FrozenCustomInvariantRegistry::from_registrations(self.custom_invariants.clone())
                .map_err(Denial::CustomInvariant)?;
        Ok(schema_contract_runtime)
    }

    fn build_commit_strategy_subsystem(
        &self,
        config: &super::RelationalRuntimeConfig,
    ) -> Result<CommitStrategiesSubsystem, Denial> {
        let mut subsystem = <CommitStrategiesSubsystem as RuntimeSubsystem>::new(&());
        subsystem.registry = FrozenCommitStrategyRegistry::from_registrations(
            config.commit_strategies.registrations.clone(),
        )
        .map_err(Denial::CommitStrategy)?;
        subsystem.executors = FrozenCommitStrategyExecutorRegistry::from_registrations(
            self.commit_strategy_executors.clone(),
            &subsystem.registry,
        )
        .map_err(Denial::CommitStrategyExecutor)?;
        Ok(subsystem)
    }

    fn build_publication_subsystem(&self) -> PublicationSubsystem {
        let mut publication = <PublicationSubsystem as RuntimeSubsystem>::new(&());
        if let Some(post_commit_consumer) = &self.post_commit_consumer {
            publication.post_commit_consumer = std::sync::Arc::clone(post_commit_consumer);
        }
        publication
    }

    fn build_runtime_services(&self) -> RuntimeServices {
        <RuntimeServices as RuntimeSubsystem>::new(&())
    }
}

impl RelationalRuntime {
    pub fn new(config: super::RelationalRuntimeConfig) -> Result<Self, Denial> {
        Self::build_from_extensions(config, RuntimeExtensions::default())
    }

    pub fn new_with_custom_invariants(
        config: super::RelationalRuntimeConfig,
        custom_invariants: Vec<CustomInvariantRegistration>,
    ) -> Result<Self, Denial> {
        Self::build_from_extensions(
            config,
            RuntimeExtensions::new(custom_invariants, Vec::new()),
        )
    }

    pub fn new_with_extensions(
        config: super::RelationalRuntimeConfig,
        custom_invariants: Vec<CustomInvariantRegistration>,
        commit_strategy_executors: Vec<CommitStrategyExecutionRegistration>,
    ) -> Result<Self, Denial> {
        Self::build_from_extensions(
            config,
            RuntimeExtensions::new(custom_invariants, commit_strategy_executors),
        )
    }

    pub(crate) fn build_from_extensions(
        config: super::RelationalRuntimeConfig,
        mut extensions: RuntimeExtensions,
    ) -> Result<Self, Denial> {
        let schema_contract_runtime =
            extensions.build_schema_contract_runtime_subsystem(&config)?;
        let commit_strategies = extensions.build_commit_strategy_subsystem(&config)?;
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
        let mut history = HistorySubsystem::from_main_cell(main_cell);
        history
            .set_runtime_instance_id(services.runtime_instance_id())
            .map_err(Denial::MainBranch)?;
        let initial_root = crate::branch::RelationalBranchRoot::empty_with_schema(
            &config.schema.registry,
            crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
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
        let visibility = <VisibilitySubsystem as RuntimeSubsystem>::new(&config);
        Ok(Self::from_state(crate::runtime::RelationalRuntimeState {
            commit_strategies,
            history,
            indexes: <IndexingSubsystem as RuntimeSubsystem>::new(&()),
            lineage: <LineageSubsystem as RuntimeSubsystem>::new(&()),
            durability,
            record_identity: <RecordIdentitySubsystem as RuntimeSubsystem>::new(&()),
            services,
            owner_lifecycle: RelationalRuntimeOwner::new(),
            publication_owner: RelationalRuntimePublicationOwner::new(),
            partitions: <StorageSubsystem as RuntimeSubsystem>::new(&()),
            visibility,
            publication: extensions.build_publication_subsystem(),
            configuration: RelationalRuntimeConfiguration::new(config, schema_contract_runtime),
        }))
    }

    pub fn fork(&self) -> Result<Self, RelationalRuntimeForkDenial> {
        let mut history = self.history.fork_snapshot()?;
        let services = RuntimeSubsystem::fork(&self.services);
        let runtime_instance_id = services.runtime_instance_id();
        history
            .bind_fork_runtime(runtime_instance_id)
            .map_err(RelationalRuntimeForkDenial::BranchIdentityRebind)?;
        let config = self.config.as_ref().clone();
        let schema_contract_runtime = RuntimeSubsystem::fork(self.schema_contract_runtime.as_ref());
        Ok(Self::from_state(crate::runtime::RelationalRuntimeState {
            configuration: RelationalRuntimeConfiguration::new(config, schema_contract_runtime),
            commit_strategies: RuntimeSubsystem::fork(&self.commit_strategies),
            partitions: self.fork_partitions(),
            visibility: RuntimeSubsystem::fork(&self.visibility),
            publication: RuntimeSubsystem::fork(&self.publication),
            history,
            indexes: RuntimeSubsystem::fork(&self.indexes),
            lineage: RuntimeSubsystem::fork(&self.lineage),
            durability: RuntimeSubsystem::fork(&self.durability),
            record_identity: RuntimeSubsystem::fork(&self.record_identity),
            services,
            owner_lifecycle: RelationalRuntimeOwner::new(),
            publication_owner: RelationalRuntimePublicationOwner::fork_from(
                &self.publication_owner,
            ),
        }))
    }
}
