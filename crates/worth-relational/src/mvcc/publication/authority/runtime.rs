//! The two existing publication preparation inputs share one owner algorithm.
use super::*;

pub(super) trait PublicationPreparationRuntime {
    fn validate_publication_request(
        &self,
        request: PublicationRequest<'_>,
    ) -> Result<ValidatedPublication, String>;

    fn prepare_publication_root(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: &crate::storage::RelationalPublishedPartitionDelta,
        envelope: &Arc<CanonicalCommitEnvelope>,
        content: &crate::content::ContentPublication,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
    ) -> Result<crate::branch::PreparedRelationalBranchRootCapture, PublicationPreparationDenial>;

    fn publication_runtime_instance_id(&self) -> u64;
    fn publication_mutation_config(&self) -> crate::config::data::MutationConfig;
}

impl PublicationPreparationRuntime for RelationalRuntime {
    fn validate_publication_request(
        &self,
        request: PublicationRequest<'_>,
    ) -> Result<ValidatedPublication, String> {
        validate_publication(self, request)
    }

    fn prepare_publication_root(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: &crate::storage::RelationalPublishedPartitionDelta,
        envelope: &Arc<CanonicalCommitEnvelope>,
        content: &crate::content::ContentPublication,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
    ) -> Result<crate::branch::PreparedRelationalBranchRootCapture, PublicationPreparationDenial>
    {
        let previous_root = selected_branch_state.root().cloned();
        self.history
            .prepare_branch_root_capture(
                execution,
                selected_branch_state.state(),
                published_partition_delta,
                previous_root.as_ref(),
                Arc::clone(envelope),
                content,
                schema_registry,
                &self.services.symbols,
            )
            .map_err(PublicationPreparationDenial::from)
    }

    fn publication_mutation_config(&self) -> crate::config::data::MutationConfig {
        crate::config::data::MutationConfig {
            cascade_delete_policy: self.config.storage.cascade_delete_policy,
            adjacency_policy: self.config.storage.adjacency_policy.clone(),
            cross_context_policy: self.config.storage.cross_context_policy,
        }
    }

    fn publication_runtime_instance_id(&self) -> u64 {
        self.history.runtime_instance_id
    }
}

impl PublicationPreparationRuntime for crate::runtime::RelationalPreparationRuntime {
    fn validate_publication_request(
        &self,
        request: PublicationRequest<'_>,
    ) -> Result<ValidatedPublication, String> {
        crate::mvcc::publication::validation::validate_prepared_publication(self, request)
    }

    fn prepare_publication_root(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected_branch_state: &SelectedRelationalBranchState,
        published_partition_delta: &crate::storage::RelationalPublishedPartitionDelta,
        envelope: &Arc<CanonicalCommitEnvelope>,
        content: &crate::content::ContentPublication,
        schema_registry: &crate::schema::data::RelationalSchemaRegistry,
    ) -> Result<crate::branch::PreparedRelationalBranchRootCapture, PublicationPreparationDenial>
    {
        let previous_root = selected_branch_state.root().cloned();
        self.history
            .prepare_branch_root_capture(
                execution,
                selected_branch_state.state(),
                published_partition_delta,
                previous_root.as_ref(),
                Arc::clone(envelope),
                content,
                schema_registry,
                &self.services.symbols,
            )
            .map_err(PublicationPreparationDenial::from)
    }

    fn publication_mutation_config(&self) -> crate::config::data::MutationConfig {
        crate::config::data::MutationConfig {
            cascade_delete_policy: self.config.storage.cascade_delete_policy,
            adjacency_policy: self.config.storage.adjacency_policy.clone(),
            cross_context_policy: self.config.storage.cross_context_policy,
        }
    }

    fn publication_runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id()
    }
}
