use worth_query_installation::facade::ApplicationSchema;

use super::{
    commit_bootstrap_rows, commit_initial_program_activation, primary_graph_denial,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphBootstrap<Schema> {
    /// Commits one bounded installation-only seed batch under the selected
    /// runtime profile. Relations must connect endpoints seeded in the same
    /// batch. The graph remains unavailable to application callers.
    /// The phase witnesses installation custody around inline Relational commits.
    pub fn commit_seed_batch(
        &mut self,
        _phase: &crate::domain_computation::primary_graph::request_execution::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if self.seed_batch_failed {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
                "a previous installation seed batch failed",
            ));
        }
        let initialization = self.take_source_initialization();
        if self.rows.is_empty() && self.entity_rows.is_empty() && self.relation_rows.is_empty() {
            if initialization.is_some() {
                self.seed_batch_failed = true;
                return Err(primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::EmptyBootstrap,
                    "initial Source construction requires a nonempty next seed batch",
                ));
            }
            return Ok(());
        }
        let result = self.commit_seed_batch_once(&_phase.execution_request(), initialization);
        if result.is_err() {
            self.seed_batch_failed = true;
        }
        result
    }

    fn commit_seed_batch_once(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        initialization: Option<
            super::super::application_query::managed_index::SourceInitializationRequest,
        >,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if let Some(seed) = self.program_activation_seed.take() {
            commit_initial_program_activation(execution, &self.graph, seed)?;
        }
        let principal_count = self.rows.len();
        let entity_count = self.entity_rows.len();
        let relation_count = self.relation_rows.len();
        let commit_id = commit_bootstrap_rows(
            execution,
            &self.graph,
            initialization,
            self.committed_principal_count,
            std::mem::take(&mut self.rows),
            std::mem::take(&mut self.entity_rows),
            std::mem::take(&mut self.relation_rows),
        )?;
        self.committed_principal_count += principal_count;
        self.committed_entity_count += entity_count;
        self.committed_relation_count += relation_count;
        self.last_seed_commit_id = Some(commit_id);
        self.pending_entity_keys.clear();
        Ok(())
    }
}
