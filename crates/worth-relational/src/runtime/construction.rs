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
use super::RelationalRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRuntimeForkDenial {
    PublicationInFlight,
    PerformedPublicationRequiresSettlement {
        commit_id: crate::history::data::CommitId,
    },
    IdentityCapacityExhausted,
    CanonicalInventoryInvalid,
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeExtensions {
    custom_invariants: Vec<CustomInvariantRegistration>,
    commit_strategy_executors: Vec<CommitStrategyExecutionRegistration>,
    post_commit_consumer: Option<std::sync::Arc<dyn crate::publication::PostCommitConsumer>>,
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
        }
    }

    pub(crate) fn with_post_commit_consumer(
        mut self,
        post_commit_consumer: std::sync::Arc<dyn crate::publication::PostCommitConsumer>,
    ) -> Self {
        self.post_commit_consumer = Some(post_commit_consumer);
        self
    }

    fn build_schema_contract_runtime_subsystem(
        &self,
        config: &super::RelationalRuntimeConfig,
    ) -> SchemaContractRuntimeSubsystem {
        let mut schema_contract_runtime =
            <SchemaContractRuntimeSubsystem as RuntimeSubsystem>::new(config);
        schema_contract_runtime.custom_invariant_registries =
            FrozenCustomInvariantRegistry::from_registrations(self.custom_invariants.clone())
                .expect(
                    "custom invariant registrations must have unique semantic identities per execution point at runtime construction",
                );
        schema_contract_runtime
    }

    fn build_commit_strategy_subsystem(
        &self,
        config: &super::RelationalRuntimeConfig,
    ) -> CommitStrategiesSubsystem {
        let mut commit_strategy_subsystem =
            <CommitStrategiesSubsystem as RuntimeSubsystem>::new(&());
        commit_strategy_subsystem.registry = FrozenCommitStrategyRegistry::from_registrations(
            config.commit_strategies.registrations.clone(),
        )
        .expect(
            "commit strategy registrations must have unique runtime identities and persistent names at runtime construction",
        );
        commit_strategy_subsystem.executors =
            FrozenCommitStrategyExecutorRegistry::from_registrations(
                self.commit_strategy_executors.clone(),
                &commit_strategy_subsystem.registry,
            )
            .expect(
                "commit strategy executors must bind to registered strategy descriptors without digest drift at runtime construction",
            );
        commit_strategy_subsystem
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
    pub fn new(config: super::RelationalRuntimeConfig) -> Self {
        Self::build_from_extensions(config, RuntimeExtensions::default())
    }

    pub fn new_with_custom_invariants(
        config: super::RelationalRuntimeConfig,
        custom_invariants: Vec<CustomInvariantRegistration>,
    ) -> Self {
        Self::build_from_extensions(
            config,
            RuntimeExtensions::new(custom_invariants, Vec::new()),
        )
    }

    pub fn new_with_extensions(
        config: super::RelationalRuntimeConfig,
        custom_invariants: Vec<CustomInvariantRegistration>,
        commit_strategy_executors: Vec<CommitStrategyExecutionRegistration>,
    ) -> Self {
        Self::build_from_extensions(
            config,
            RuntimeExtensions::new(custom_invariants, commit_strategy_executors),
        )
    }

    pub(crate) fn build_from_extensions(
        config: super::RelationalRuntimeConfig,
        extensions: RuntimeExtensions,
    ) -> Self {
        let services = extensions.build_runtime_services();
        services
            .symbols
            .initialize_configuration_snapshot(config.identity.symbol_table.clone());
        let mut history = <HistorySubsystem as RuntimeSubsystem>::new(&config.history.main_branch);
        history.set_runtime_instance_id(services.runtime_instance_id());
        let initial_root = crate::branch::RelationalBranchRoot::empty_with_schema(
            &config.schema.registry,
            crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
        );
        history
            .install_branch_root(&config.history.main_branch, initial_root)
            .expect("the configured main branch fits the initial retention budget");
        let schema_contract_runtime = extensions.build_schema_contract_runtime_subsystem(&config);
        let commit_strategies = extensions.build_commit_strategy_subsystem(&config);
        let durability = <DurabilitySubsystem as RuntimeSubsystem>::new(&config);
        let visibility = <VisibilitySubsystem as RuntimeSubsystem>::new(&config);
        Self::from_state(crate::runtime::RelationalRuntimeState {
            commit_strategies,
            history,
            indexes: <IndexingSubsystem as RuntimeSubsystem>::new(&()),
            lineage: <LineageSubsystem as RuntimeSubsystem>::new(&()),
            durability,
            record_identity: <RecordIdentitySubsystem as RuntimeSubsystem>::new(&()),
            services,
            owner_lifecycle: RelationalRuntimeOwner::new(),
            publication_owner: RelationalRuntimePublicationOwner::new(
                config.publication.policy.max_prepared_root_bytes,
            ),
            partitions: <StorageSubsystem as RuntimeSubsystem>::new(&()),
            visibility,
            publication: extensions.build_publication_subsystem(),
            configuration: RelationalRuntimeConfiguration::new(config, schema_contract_runtime),
        })
    }

    pub fn fork(&self) -> Result<Self, RelationalRuntimeForkDenial> {
        let mut history = self.history.fork_snapshot()?;
        let services = RuntimeSubsystem::fork(&self.services);
        let runtime_instance_id = services.runtime_instance_id();
        history.bind_fork_runtime(runtime_instance_id);
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
