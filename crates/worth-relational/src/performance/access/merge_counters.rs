use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    pub(crate) fn count_merge_history_ancestry_traversal(&self, nodes_visited: usize) {
        self.counters.count(|counters| {
            counters.merge_history_ancestry_traversals += 1;
            counters.merge_history_ancestry_nodes_visited += nodes_visited;
        });
    }

    pub(crate) fn count_merge_history_parent_comparisons(&self, comparisons: usize) {
        self.counters.count(|counters| {
            counters.merge_history_parent_comparisons += comparisons;
        });
    }

    pub(crate) fn count_merge_history_replay_planning(
        &self,
        nodes_visited: usize,
        parent_checks: usize,
    ) {
        self.counters.count(|counters| {
            counters.merge_history_replay_planning_nodes_visited += nodes_visited;
            counters.merge_history_replay_parent_checks += parent_checks;
        });
    }

    pub(crate) fn count_merge_history_durability_validation(
        &self,
        nodes_visited: usize,
        parent_checks: usize,
    ) {
        self.counters.count(|counters| {
            counters.merge_history_durability_validation_nodes_visited += nodes_visited;
            counters.merge_history_durability_parent_checks += parent_checks;
        });
    }

    pub(crate) fn count_merge_planning_request(
        &self,
        schema_kinds: usize,
        target_commits: usize,
        source_commits: usize,
        target_records: usize,
        source_records: usize,
    ) {
        self.counters.count(|counters| {
            counters.merge_planning_requests += 1;
            counters.merge_planning_schema_kinds_snapshotted += schema_kinds;
            counters.merge_planning_target_commits_scoped += target_commits;
            counters.merge_planning_source_commits_scoped += source_commits;
            counters.merge_planning_target_records_scoped += target_records;
            counters.merge_planning_source_records_scoped += source_records;
        });
    }

    pub(crate) fn count_merge_identity_discovery(&self, candidates: usize, declarations: usize) {
        self.counters.count(|counters| {
            counters.merge_identity_candidates_discovered += candidates;
            counters.merge_identity_effective_declarations += declarations;
        });
    }

    pub(crate) fn count_merge_identity_target_indexing(&self, scanned: usize, indexed: usize) {
        self.counters.count(|counters| {
            counters.merge_identity_target_records_scanned += scanned;
            counters.merge_identity_target_records_indexed += indexed;
        });
    }

    pub(crate) fn count_merge_conflict_classification(&self, records: usize) {
        self.counters.count(|counters| {
            counters.merge_conflict_records_classified += records;
        });
    }

    pub(crate) fn count_merge_causal_annotation(&self, records: usize) {
        self.counters.count(|counters| {
            counters.merge_causal_records_annotated += records;
        });
    }

    pub(crate) fn count_merge_policy_resolution(&self, records: usize) {
        self.counters.count(|counters| {
            counters.merge_policy_records_resolved += records;
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn count_merge_policy_value_lookup(
        &self,
        source_state_hits: u64,
        target_state_hits: u64,
        base_state_hits: u64,
        base_patch_authority_hits: u64,
        missing_ancestor_basis: u64,
        missing_visible_state: u64,
        invalid_shape: u64,
    ) {
        self.counters.count(|counters| {
            counters.merge_policy_value_source_state_hits += source_state_hits as usize;
            counters.merge_policy_value_target_state_hits += target_state_hits as usize;
            counters.merge_policy_value_base_state_hits += base_state_hits as usize;
            counters.merge_policy_value_base_patch_authority_hits +=
                base_patch_authority_hits as usize;
            counters.merge_policy_value_missing_ancestor_basis += missing_ancestor_basis as usize;
            counters.merge_policy_value_missing_visible_state += missing_visible_state as usize;
            counters.merge_policy_value_invalid_shape += invalid_shape as usize;
        });
    }

    pub(crate) fn count_merge_lowering(&self, lowered_records: usize, decision_log_width: usize) {
        self.counters.count(|counters| {
            counters.merge_lowered_records_emitted += lowered_records;
            counters.merge_decision_log_width += decision_log_width;
        });
    }

    pub(crate) fn count_merge_planning_elapsed(&self, elapsed_nanos: u128) {
        self.counters.count(|counters| {
            counters.merge_planning_elapsed_nanos += elapsed_nanos;
        });
    }

    pub(crate) fn count_merge_topology_region_detection(
        &self,
        relation_candidates: usize,
        endpoint_incidences: usize,
        region_conflicts: usize,
        region_records_escalated: usize,
    ) {
        self.counters.count(|counters| {
            counters.merge_topology_relation_candidates_scoped += relation_candidates;
            counters.merge_topology_endpoint_incidences_scoped += endpoint_incidences;
            counters.merge_topology_region_conflicts_detected += region_conflicts;
            counters.merge_topology_region_records_escalated += region_records_escalated;
        });
    }

    pub(crate) fn count_merge_execution_verification_request(&self) {
        self.counters.count(|counters| {
            counters.merge_execution_verification_requests += 1;
        });
    }

    pub(crate) fn count_merge_execution_branch_head_checks(&self, checks: usize) {
        self.counters.count(|counters| {
            counters.merge_execution_branch_head_checks += checks;
        });
    }

    pub(crate) fn count_merge_execution_merge_base_checks(&self, checks: usize) {
        self.counters.count(|counters| {
            counters.merge_execution_merge_base_checks += checks;
        });
    }

    pub(crate) fn count_merge_execution_schema_snapshot_kinds(&self, kinds: usize) {
        self.counters.count(|counters| {
            counters.merge_execution_schema_kinds_snapshotted += kinds;
        });
    }

    pub(crate) fn count_merge_execution_compiled_plan_digest_checks(&self, checks: usize) {
        self.counters.count(|counters| {
            counters.merge_execution_compiled_plan_digest_checks += checks;
        });
    }

    pub(crate) fn count_merge_execution_attempt(&self) {
        self.counters.count(|counters| {
            counters.merge_execution_attempts += 1;
        });
    }

    pub(crate) fn count_merge_execution_request(
        &self,
        admitted_records: usize,
        emitted_mutation_intents: usize,
    ) {
        self.counters.count(|counters| {
            counters.merge_execution_requests += 1;
            counters.merge_execution_records_admitted += admitted_records;
            counters.merge_execution_mutation_intents_emitted += emitted_mutation_intents;
        });
    }
}
