use super::RelationalRuntime;

impl RelationalRuntime {
    /// Read-only candidate preparation observation for integration proof.
    pub fn custom_invariant_candidate_input_counters(
        &self,
    ) -> crate::runtime::RelationalCandidateInputCounters {
        let counters = self.performance_access().complexity_counters_snapshot();
        crate::runtime::RelationalCandidateInputCounters {
            entity_reads: counters.custom_invariant_candidate_entity_reads,
            relation_reads: counters.custom_invariant_candidate_relation_reads,
            entity_aspect_reads: counters.custom_invariant_candidate_entity_aspect_reads,
            relation_aspect_reads: counters.custom_invariant_candidate_relation_aspect_reads,
            adjacency_gathers: counters.custom_invariant_candidate_adjacency_gathers,
            reuse_hits: counters.custom_invariant_candidate_reuse_hits,
        }
    }

    pub(crate) fn retention_cost_counters(
        &self,
    ) -> crate::history::retention::RelationalRetentionCostCounters {
        self.history.retention_cost_counters()
    }

    /// Capture the runtime's current public configuration, including symbols
    /// admitted concurrently through shared preparation ports.
    pub fn config(&self) -> crate::runtime::RelationalRuntimeConfig {
        let mut snapshot = self.config.as_ref().clone();
        snapshot.identity.symbol_table = self.services.symbols.configuration_snapshot();
        snapshot
    }

    pub fn commit_strategy_registry(
        &self,
    ) -> &crate::commit_strategies::FrozenCommitStrategyRegistry {
        &self.commit_strategies.registry
    }

    pub(crate) fn commit_strategy_executor_registry(
        &self,
    ) -> &crate::commit_strategies::FrozenCommitStrategyExecutorRegistry {
        &self.commit_strategies.executors
    }

    pub fn commit_strategies(
        &self,
    ) -> crate::commit_strategies::facade::CommitStrategiesFacade<'_> {
        crate::commit_strategies::facade::CommitStrategiesFacade::new(self)
    }

    pub fn commit_strategies_authority(
        &self,
    ) -> crate::commit_strategies::facade::CommitStrategiesAuthorityFacade {
        crate::commit_strategies::facade::CommitStrategiesAuthorityFacade::new()
    }

    pub fn phase4_reference_cost_counters(
        &self,
    ) -> crate::runtime::RelationalPhase4ReferenceCostCounters {
        self.history.phase4_costs()
    }

    pub fn branch_basis_cost_counters(&self) -> crate::branch::RelationalBranchBasisCostCounters {
        let mut counters = self.services.instrumentation.basis_counters();
        let (entries, key_lookups, mutations) = self.history.basis_registry_metrics();
        counters.retained_basis_registry_entries = entries;
        counters.retained_basis_registry_key_lookups = key_lookups;
        counters.retained_basis_registry_mutations = mutations;
        counters
    }

    pub(crate) fn branch_sharing_cost_counters_for_branch(
        &self,
        branch_id: &crate::history::data::BranchId,
    ) -> crate::runtime::RelationalBranchSharingCostCounters {
        self.history.sharing_costs_for_branch(branch_id)
    }

    pub(crate) fn branch_sharing_cost_counters_for_branches(
        &self,
        branch_ids: &[crate::history::data::BranchId],
    ) -> crate::runtime::RelationalBranchSharingCostCounters {
        let mut total = crate::runtime::RelationalBranchSharingCostCounters::default();
        for branch_id in branch_ids {
            let costs = self.branch_sharing_cost_counters_for_branch(branch_id);
            total.branch_population_scans = total
                .branch_population_scans
                .saturating_add(costs.branch_population_scans);
            total.snapshot_root_reads = total
                .snapshot_root_reads
                .saturating_add(costs.snapshot_root_reads);
            total.transaction_validation_attempts = total
                .transaction_validation_attempts
                .saturating_add(costs.transaction_validation_attempts);
            total.retained_history_head_lookups = total
                .retained_history_head_lookups
                .saturating_add(costs.retained_history_head_lookups);
            total.candidate_preparations = total
                .candidate_preparations
                .saturating_add(costs.candidate_preparations);
            total.candidate_discards = total
                .candidate_discards
                .saturating_add(costs.candidate_discards);
            total.publication_attempts = total
                .publication_attempts
                .saturating_add(costs.publication_attempts);
            total.shared_root_acquisitions = total
                .shared_root_acquisitions
                .saturating_add(costs.shared_root_acquisitions);
            total.copied_truth_bytes = total
                .copied_truth_bytes
                .saturating_add(costs.copied_truth_bytes);
            total.copied_commit_envelopes = total
                .copied_commit_envelopes
                .saturating_add(costs.copied_commit_envelopes);
            total.fork_materialized_entity_count = total
                .fork_materialized_entity_count
                .saturating_add(costs.fork_materialized_entity_count);
            total.fork_materialized_relation_count = total
                .fork_materialized_relation_count
                .saturating_add(costs.fork_materialized_relation_count);
            total.fork_materialized_authoritative_bytes = total
                .fork_materialized_authoritative_bytes
                .saturating_add(costs.fork_materialized_authoritative_bytes);
            total.publication_touched_region_count = total
                .publication_touched_region_count
                .saturating_add(costs.publication_touched_region_count);
            total.publication_reused_region_count = total
                .publication_reused_region_count
                .saturating_add(costs.publication_reused_region_count);
            total.publication_persistent_index_path_nodes = total
                .publication_persistent_index_path_nodes
                .saturating_add(costs.publication_persistent_index_path_nodes);
            total.publication_new_authoritative_bytes = total
                .publication_new_authoritative_bytes
                .saturating_add(costs.publication_new_authoritative_bytes);
            total.publication_content_values_hashed = total
                .publication_content_values_hashed
                .saturating_add(costs.publication_content_values_hashed);
            total.reclaimable_unique_bytes = total
                .reclaimable_unique_bytes
                .saturating_add(costs.reclaimable_unique_bytes);
        }
        total
    }

    pub(crate) fn branch_population_scan_count(&self) -> u64 {
        self.history.branch_population_scan_count()
    }
}
