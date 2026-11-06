use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphBootstrap<Schema> {
    /// The phase witnesses the caller's scope around inline Relational publication.
    pub(in crate::domain_computation::primary_graph) fn publish_with_resource_support(
        mut self,

        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &mut WorthQueryExecutionRuntime,
        authority: &WorthQueryExecutionInstallationAuthority,
    ) -> Result<
        (
            WorthQueryPrimaryGraphPublication,
            super::super::provider::WorthQueryPrimaryGraphResourceSupport,
        ),
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        self.validate_publication_target(runtime, authority)?;
        let publication = home_publication::publish(&mut self, phase)?;
        runtime.install_primary_graph(self.graph);
        Ok((publication, self.resource_support))
    }

    /// Borrowed Home preparation retains the graph and acknowledged seed state
    /// if publication stops. The caller cannot retry a failed seed batch.
    pub(in crate::domain_computation::primary_graph) fn publish_home_bootstrap_rows(
        &mut self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<WorthQueryPrimaryGraphPublication, WorthQueryPrimaryGraphInstallationDenial> {
        let request = phase.execution_request();
        let execution = &request;
        if self.seed_batch_failed {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
                "a previous installation seed batch failed",
            ));
        }
        if let Some(publication) = &self.recovered_publication {
            return Ok(publication.clone());
        }
        if self.rows.is_empty() && self.committed_principal_count == 0 {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::EmptyBootstrap,
                "at least one application principal binding is required",
            ));
        }
        let initialization = self.take_source_initialization();
        if initialization.is_some()
            && self.rows.is_empty()
            && self.typed_rows.entity_rows.is_empty()
            && self.typed_rows.relation_rows.is_empty()
        {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::EmptyBootstrap,
                "initial Source construction requires a nonempty next seed batch",
            ));
        }
        let row_count = self.committed_principal_count + self.rows.len();
        let entity_count = self.committed_entity_count + self.typed_rows.entity_rows.len();
        let relation_count = self.committed_relation_count + self.typed_rows.relation_rows.len();
        let principal_identity_index_count = self
            .graph
            .layout
            .principal_bindings()
            .map(|(_, binding)| binding.index_id)
            .collect::<BTreeSet<_>>()
            .len();
        let application_equality_index_count = self
            .graph
            .layout
            .equality_index_ids()
            .collect::<BTreeSet<_>>()
            .len();
        let index_ids = self.graph.integration_handle().primary_index_ids.to_vec();
        self.seed_batch_failed = true;
        if let Some(seed) = self.program_activation_seed.take() {
            commit_initial_program_activation(execution, &self.graph, seed)?;
        }
        let commit_id = if self.rows.is_empty()
            && self.typed_rows.entity_rows.is_empty()
            && self.typed_rows.relation_rows.is_empty()
        {
            self.last_seed_commit_id
                .expect("a principal seed batch was committed")
        } else {
            commit_bootstrap_rows(
                execution,
                &self.graph,
                initialization,
                self.committed_principal_count,
                std::mem::take(&mut self.rows),
                std::mem::take(&mut self.typed_rows.entity_rows),
                std::mem::take(&mut self.typed_rows.relation_rows),
            )?
        };
        self.last_seed_commit_id = Some(commit_id);
        build_identity_indexes(execution, &self.graph, commit_id, &index_ids)?;
        let binding_identity = self.graph.binding_identity().clone();
        let publication = WorthQueryPrimaryGraphPublication {
            binding_identity,
            principal_binding_count: row_count,
            identity_index_count: principal_identity_index_count,
            application_equality_index_count,
            policy_entity_count: entity_count,
            policy_relation_count: relation_count,
            bootstrap_commit_id: commit_id,
        };
        self.seed_batch_failed = false;
        Ok(publication)
    }
}
