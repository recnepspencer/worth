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
        let configuration = self.operation_configuration();
        let mut snapshot = configuration.config.as_ref().clone();
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

    pub(crate) fn try_branch_sharing_cost_counters_for_branches(
        &self,
        branch_ids: &[crate::history::data::BranchId],
    ) -> Result<
        crate::runtime::RelationalBranchSharingCostCounters,
        crate::storage::substrate::StorageAllocationOverflow,
    > {
        use crate::storage::substrate::StorageAllocationOverflow;
        let mut total = crate::runtime::RelationalBranchSharingCostCounters::default();
        for branch_id in branch_ids {
            let costs = self.history.try_sharing_costs_for_branch(branch_id)?;
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
                .checked_add(costs.copied_truth_bytes)
                .ok_or(StorageAllocationOverflow)?;
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
                .checked_add(costs.fork_materialized_authoritative_bytes)
                .ok_or(StorageAllocationOverflow)?;
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
                .checked_add(costs.publication_new_authoritative_bytes)
                .ok_or(StorageAllocationOverflow)?;
            total.publication_new_root_authoritative_bytes = total
                .publication_new_root_authoritative_bytes
                .checked_add(costs.publication_new_root_authoritative_bytes)
                .ok_or(StorageAllocationOverflow)?;
            total.publication_content_values_hashed = total
                .publication_content_values_hashed
                .saturating_add(costs.publication_content_values_hashed);
            total.reclaimable_unique_bytes = total
                .reclaimable_unique_bytes
                .checked_add(costs.reclaimable_unique_bytes)
                .ok_or(StorageAllocationOverflow)?;
        }
        Ok(total)
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
            total.publication_new_root_authoritative_bytes = total
                .publication_new_root_authoritative_bytes
                .saturating_add(costs.publication_new_root_authoritative_bytes);
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

    /// Returns the persisted lineage identity bound to a committed relation, if
    /// any.
    pub fn relation_lineage(
        &self,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<crate::identity::data::LineageId> {
        self.lineage.relation_lineage(relation_id)
    }

    /// Returns the installed `(subject_kind, subject_name)` for `scheme_id`
    /// from Relational's own schema registry, if installed.
    pub fn persistent_name_scheme_subject(
        &self,
        scheme_id: &str,
    ) -> Option<(crate::lineage::data::PersistentNameSubjectKind, String)> {
        self.config
            .schema
            .registry
            .persistent_name_scheme(scheme_id)
            .map(|scheme| (scheme.subject_kind(), scheme.subject_name().to_owned()))
    }

    /// Whether installed scheme `scheme_id` mints structural identities.
    pub fn persistent_name_scheme_structural(&self, scheme_id: &str) -> bool {
        let registry = &self.config.schema.registry;
        let scheme = registry.persistent_name_scheme(scheme_id);
        scheme.is_some_and(|scheme| scheme.structural_identity())
    }

    /// The installed scheme governing `subject_name` of class `subject_kind`.
    pub fn persistent_name_scheme_id(
        &self,
        subject_kind: crate::lineage::data::PersistentNameSubjectKind,
        subject_name: &str,
    ) -> Option<String> {
        let schemes = self.config.schema.registry.persistent_name_schemes();
        schemes
            .iter()
            .find(|scheme| {
                scheme.subject_kind() == subject_kind && scheme.subject_name() == subject_name
            })
            .map(|scheme| scheme.scheme_id().to_owned())
    }

    /// Read-only view of every currently live persistent-name holder derived
    /// from Relational's committed lineage events on `branch_id`. Returns a
    /// typed error instead of panicking when a cold naming page is damaged or
    /// the branch's naming basis is unavailable.
    pub fn branch_live_persistent_names(
        &self,
        branch_id: &crate::history::data::BranchId,
    ) -> Result<
        std::collections::BTreeMap<
            crate::lineage::data::PersistentNameIdentity,
            crate::lineage::data::PersistentNameHolder,
        >,
        worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::NamingBasisUnavailable,
    >{
        self.lineage.branch_live_persistent_names(branch_id)
    }

    /// Page-cache observation of the currently selected checkpoint's naming
    /// store, when cold-backed.
    pub fn cold_naming_observation(
        &self,
    ) -> Option<worth_relational_storage::internal_port::storage::residency::chunked::page_cache::PageCacheObservation>
{
        self.durability
            .checkpoint_selection()
            .snapshot()
            .map(|dir| dir.naming_state().observation())
    }

    /// Freezes and seals a read-only `CHK24` naming page snapshot of all
    /// committed branches, returning `(page_count, new_member_bytes)`.
    pub fn seal_naming_snapshot(&self) -> Result<(usize, usize), String> {
        let (pages, bytes, _) = self.seal_naming_snapshot_with_visits()?;
        Ok((pages, bytes))
    }

    /// Freezes and seals a read-only `CHK24` naming page snapshot of all
    /// committed branches, returning `(page_count, new_member_bytes, diff_nodes_visited)`.
    pub fn seal_naming_snapshot_with_visits(&self) -> Result<(usize, usize, usize), String> {
        let committed: Vec<_> = self
            .history
            .branch_cells_snapshot()
            .into_iter()
            .filter(|cell| {
                matches!(
                    cell.observation.target(),
                    worth_foundational::FoundationalBranchTarget::Basis(_)
                )
            })
            .map(|cell| cell.branch_id)
            .collect();
        let frozen = self.lineage.freeze_naming();
        let mut sink = worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers::default();
        let (state, visited) = frozen
            .seal_with_diff_visits(&committed, &mut sink)
            .map_err(|e| format!("{e:?}"))?;
        let bytes: usize = sink.members.iter().map(|(_, b)| b.len()).sum();
        Ok((state.page_members().len(), bytes, visited))
    }
}
