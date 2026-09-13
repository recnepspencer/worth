use crate::data::aspect::{Aspect, AspectVersion};
use crate::data::dependency::DependencySnapshotId;
use crate::data::graph::{DependencySetId, SubscriberSetId};
use crate::data::node::NodeEvaluationConfig;
use crate::data::output::{ChangedRegion, PartitionSubscription};

use super::NodeEntry;

#[cfg_attr(not(test), allow(dead_code))]
impl NodeEntry {
    /// The current aspect versions.
    pub fn get_aspect_version(&self) -> AspectVersion {
        self.hot.aspect_version_header.global()
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn get_partitioned_aspect_version(&self, scope: &PartitionSubscription) -> AspectVersion {
        self.warm
            .aspect_version_overrides
            .scoped_or_global(scope, self.hot.aspect_version_header.global())
    }

    pub fn version_for_scope(&self, aspect: Aspect, scope: Option<&PartitionSubscription>) -> u64 {
        self.warm.aspect_version_overrides.version_for_scope(
            aspect,
            scope,
            self.hot.aspect_version_header.global(),
        )
    }

    /// Set the aspect version after evaluation.
    pub fn set_aspect_version(&mut self, version: AspectVersion) {
        self.hot.aspect_version_header.set_global(version);
        self.warm.aspect_version_overrides.set_global(version);
        self.hot
            .aspect_version_header
            .set_has_partition_overrides(self.warm.aspect_version_overrides.has_overrides());
    }

    pub fn apply_aspect_version(
        &mut self,
        version: AspectVersion,
        changed_regions: &[ChangedRegion],
    ) {
        self.hot.aspect_version_header.set_global(version);
        self.warm
            .aspect_version_overrides
            .apply_evaluation(version, changed_regions);
        self.hot
            .aspect_version_header
            .set_has_partition_overrides(self.warm.aspect_version_overrides.has_overrides());
    }

    /// Graph-owned dependency set handle.
    pub fn get_dependencies_id(&self) -> DependencySetId {
        self.hot.dependencies_id
    }

    /// Replace the dependency set handle.
    pub fn set_dependencies_id(&mut self, dependencies_id: DependencySetId) {
        self.hot.dependencies_id = dependencies_id;
    }

    /// Graph-owned subscriber set handle.
    pub fn get_subscribers_id(&self) -> SubscriberSetId {
        self.hot.subscribers_id
    }

    /// Replace the subscriber set handle.
    pub fn set_subscribers_id(&mut self, subscribers_id: SubscriberSetId) {
        self.hot.subscribers_id = subscribers_id;
    }

    /// The graph-owned dependency snapshot handle from the last clean evaluation.
    pub fn get_dep_snapshot_id(&self) -> DependencySnapshotId {
        self.hot.dep_snapshot_id
    }

    /// Replace the dependency snapshot handle.
    pub fn set_dep_snapshot_id(&mut self, snapshot_id: DependencySnapshotId) {
        self.hot.dep_snapshot_id = snapshot_id;
    }

    /// Whether this node is tombstoned.
    pub fn is_tombstoned(&self) -> bool {
        self.definition.tombstoned
    }

    /// Mark this node as tombstoned.
    #[cfg(test)]
    pub fn set_tombstoned(&mut self, tombstoned: bool) {
        self.definition.tombstoned = tombstoned;
    }

    /// Per-node evaluation policy descriptor.
    pub fn get_eval_config(&self) -> &NodeEvaluationConfig {
        &self.definition.eval_config
    }

    /// Replace per-node evaluation policy descriptor.
    pub fn set_eval_config(&mut self, config: NodeEvaluationConfig) {
        self.definition.eval_config = config.upgrade_legacy_output_equivalence();
    }

    pub(crate) const fn conditional_contract_generation(&self) -> u64 {
        self.definition.conditional_contract_generation
    }

    pub(crate) const fn conditional_contract_occurrence(&self) -> u64 {
        self.definition.conditional_contract_occurrence
    }

    pub(crate) fn install_conditional_contract_occurrence(
        &mut self,
        occurrence: u64,
    ) -> Option<u64> {
        let next = self
            .definition
            .conditional_contract_generation
            .checked_add(1)?;
        self.definition.conditional_contract_generation = next;
        self.definition.conditional_contract_occurrence = occurrence;
        Some(next)
    }

    pub(crate) const fn pending_cause_set_id(
        &self,
    ) -> crate::data::graph::storage::invalidation_causes::PendingCauseSetId {
        self.hot.pending_cause_set_id
    }

    pub(crate) fn set_pending_cause_set_id(
        &mut self,
        id: crate::data::graph::storage::invalidation_causes::PendingCauseSetId,
    ) {
        self.hot.pending_cause_set_id = id;
    }

    pub(crate) const fn dependency_revision(
        &self,
    ) -> crate::data::proof::invalidation::binding::DependencyRevision {
        self.hot.dependency_revision
    }

    pub(crate) fn pending_dependency_revalidation(
        &self,
    ) -> Option<&crate::data::proof::invalidation::binding::PendingDependencyRevalidation> {
        self.warm.pending_dependency_revalidation.as_ref()
    }

    pub(crate) fn direct_invalidation_basis(
        &self,
    ) -> Option<&crate::data::proof::invalidation::source_seed::DirectInvalidationBasis> {
        self.warm.direct_invalidation_basis.as_ref()
    }

    pub(crate) const fn direct_invalidation_generation(&self) -> u64 {
        self.warm.direct_invalidation_generation
    }

    pub(crate) fn dirty_partition_scope_payload(&self) -> &[(Aspect, PartitionSubscription)] {
        self.warm.dirty_partition_scope_payload.as_slice()
    }

    pub(crate) fn mark_pending_dependency_revalidation(
        &mut self,
        producers: impl IntoIterator<Item = crate::data::handle::NodeId>,
    ) {
        self.warm.pending_dependency_revalidation = Some(
            crate::data::proof::invalidation::binding::PendingDependencyRevalidation::new(
                self.hot.dependency_revision,
                producers,
            ),
        );
    }

    pub(crate) fn mark_pending_structural_revalidation(
        &mut self,
        producers: impl IntoIterator<Item = crate::data::handle::NodeId>,
    ) {
        self.warm.pending_dependency_revalidation = Some(
            crate::data::proof::invalidation::binding::PendingDependencyRevalidation::structural(
                self.hot.dependency_revision,
                producers,
            ),
        );
    }

    pub(crate) fn resolve_pending_dependency_producer(
        &mut self,
        producer: crate::data::handle::NodeId,
    ) -> bool {
        let Some(pending) = self.warm.pending_dependency_revalidation.as_mut() else {
            return false;
        };
        pending.resolve_producer(producer);
        if pending.is_resolved() && !pending.requires_structural_recompute() {
            self.warm.pending_dependency_revalidation = None;
            return true;
        }
        false
    }
}
