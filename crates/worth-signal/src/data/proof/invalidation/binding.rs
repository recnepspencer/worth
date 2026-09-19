use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;

mod retained_charge;

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub(crate) struct DependencyRevision(pub(crate) u64);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct OutputCommitOrdinal(pub(crate) u64);

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct DependencyCauseKey {
    pub(crate) graph_instance: u64,
    pub(crate) consumer: NodeId,
    pub(crate) dependency_revision: DependencyRevision,
    pub(crate) producer: NodeId,
    pub(crate) aspect: Aspect,
    pub(crate) edge_scope: Option<PartitionSubscription>,
}

worth_proof::binding_axes! {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub(crate) struct DependencyCauseBindingAxes {
        pub(crate) graph_instance: u64 => GraphInstance,
        pub(crate) consumer: NodeId => Consumer,
        pub(crate) dependency_revision: DependencyRevision => DependencyRevision,
        pub(crate) producer: NodeId => Producer,
        pub(crate) aspect: Aspect => Aspect,
        pub(crate) edge_scope: Option<PartitionSubscription> => EdgeScope,
        pub(crate) cached_version: u64 => CachedVersion,
        pub(crate) output_commit_ordinal: OutputCommitOrdinal => OutputCommitOrdinal,
        pub(crate) committed_version: u64 => CommittedVersion,
    }
    drift pub(crate) enum DependencyCauseBindingDrift;
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResolvedDependencyCause {
    pub(crate) key: DependencyCauseKey,
    pub(crate) binding_axes: DependencyCauseBindingAxes,
    pub(crate) changed_scopes: crate::data::proof::PartitionScopeSet,
}

impl ResolvedDependencyCause {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        graph_instance: u64,
        consumer: NodeId,
        dependency_revision: DependencyRevision,
        producer: NodeId,
        aspect: Aspect,
        edge_scope: Option<PartitionSubscription>,
        cached_version: u64,
        output_commit_ordinal: OutputCommitOrdinal,
        committed_version: u64,
        changed_scopes: crate::data::proof::PartitionScopeSet,
    ) -> Self {
        let key = DependencyCauseKey {
            graph_instance,
            consumer,
            dependency_revision,
            producer,
            aspect,
            edge_scope: edge_scope.clone(),
        };
        let binding_axes = DependencyCauseBindingAxes {
            graph_instance,
            consumer,
            dependency_revision,
            producer,
            aspect,
            edge_scope,
            cached_version,
            output_commit_ordinal,
            committed_version,
        };
        Self {
            key,
            binding_axes,
            changed_scopes,
        }
    }

    #[cfg(test)]
    pub(crate) fn binding(&self) -> worth_proof::Binding<DependencyCauseBindingAxes> {
        worth_proof::Binding::new(self.binding_axes.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PendingDependencyRevalidation {
    dependency_revision: DependencyRevision,
    unresolved_producers: Vec<NodeId>,
    #[serde(default)]
    requires_structural_recompute: bool,
}

impl PendingDependencyRevalidation {
    pub(crate) fn new(
        dependency_revision: DependencyRevision,
        producers: impl IntoIterator<Item = NodeId>,
    ) -> Self {
        let mut unresolved_producers = producers.into_iter().collect::<Vec<_>>();
        unresolved_producers.sort_unstable();
        unresolved_producers.dedup();
        Self {
            dependency_revision,
            unresolved_producers,
            requires_structural_recompute: false,
        }
    }

    pub(crate) fn structural(
        dependency_revision: DependencyRevision,
        producers: impl IntoIterator<Item = NodeId>,
    ) -> Self {
        let mut pending = Self::new(dependency_revision, producers);
        pending.requires_structural_recompute = true;
        pending
    }

    pub(crate) const fn dependency_revision(&self) -> DependencyRevision {
        self.dependency_revision
    }

    pub(crate) fn unresolved_producers(&self) -> &[NodeId] {
        &self.unresolved_producers
    }

    pub(crate) const fn requires_structural_recompute(&self) -> bool {
        self.requires_structural_recompute
    }

    pub(crate) fn resolve_producer(&mut self, producer: NodeId) {
        self.unresolved_producers
            .retain(|candidate| *candidate != producer);
    }

    pub(crate) fn is_resolved(&self) -> bool {
        self.unresolved_producers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(index: u32) -> NodeId {
        NodeId::new(index, 0)
    }

    worth_proof::binding_axis_drift_certification! {
        binding: DependencyCauseBindingAxes,
        drift: DependencyCauseBindingDrift,
        base: DependencyCauseBindingAxes {
            graph_instance: 7,
            consumer: node(2),
            dependency_revision: DependencyRevision(3),
            producer: node(1),
            aspect: Aspect::new(4),
            edge_scope: None,
            cached_version: 5,
            output_commit_ordinal: OutputCommitOrdinal(6),
            committed_version: 8,
        },
        twins: {
            graph_instance => GraphInstance = 9,
            consumer => Consumer = node(3),
            dependency_revision => DependencyRevision = DependencyRevision(4),
            producer => Producer = node(4),
            aspect => Aspect = Aspect::new(5),
            edge_scope => EdgeScope = Some(PartitionSubscription::whole_partition("rates")),
            cached_version => CachedVersion = 6,
            output_commit_ordinal => OutputCommitOrdinal = OutputCommitOrdinal(7),
            committed_version => CommittedVersion = 9,
        }
    }
}
