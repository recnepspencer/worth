use crate::data::aspect::Aspect;
use crate::data::dependency::{CanonicalDependencies, DependencyEdge, DependencySnapshot};
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::output::PartitionSubscription;
use crate::data::proof::DependencyBatchEdit;

use crate::data::graph::signal_graph::SignalGraph;

impl SignalGraph {
    pub fn set_dependencies(
        &mut self,
        node: NodeId,
        desired: impl IntoIterator<Item = DependencyEdge>,
    ) -> Result<(), SignalError> {
        let desired = CanonicalDependencies::new(desired);
        let _ = self.reconcile_dependencies(node, desired.as_slice())?;
        Ok(())
    }

    /// Installs the dependency set an evaluation of `node` has just
    /// discovered.
    ///
    /// `set_dependencies` replaces the topology under a cached value the
    /// caller knows nothing about, so it records a structural revalidation
    /// and demotes a clean node to `MaybeStale`: the next read recomputes it.
    /// Here the cached value was computed against exactly `desired`, so the
    /// edges are installed, the dependency snapshot is recorded at the
    /// sources' current versions, and a node that was `Clean` before the
    /// replacement is `Clean` after it. A node that was not clean (an
    /// invalidation landed after the evaluation) keeps its state and its
    /// pending revalidation, so the next read still recomputes it.
    ///
    /// Cost: one `set_dependencies` plus O(desired) version lookups for the
    /// snapshot.
    pub fn set_evaluated_dependencies(
        &mut self,
        node: NodeId,
        desired: impl IntoIterator<Item = DependencyEdge>,
    ) -> Result<(), SignalError> {
        let was_clean = matches!(self.get_state(node)?, NodeState::Clean);
        self.set_dependencies(node, desired)?;
        let mut snapshot = DependencySnapshot::empty();
        for edge in self.dependencies_of(node)? {
            let version = self.node_aspect_version(edge.source())?.get(edge.aspect());
            snapshot.record(
                edge.source(),
                edge.aspect(),
                version,
                edge.scope_ref().cloned(),
            );
        }
        self.set_dep_snapshot(node, snapshot)?;
        if was_clean {
            self.transition_node_clean(node)?;
        }
        Ok(())
    }

    pub fn clear_dependencies(&mut self, node: NodeId) -> Result<(), SignalError> {
        self.set_dependencies(node, std::iter::empty())
    }

    #[cfg(test)]
    pub(crate) fn edit_dependencies(
        &mut self,
        node: NodeId,
        edit: impl FnOnce(&mut Vec<DependencyEdge>),
    ) -> Result<(), SignalError> {
        let mut desired = self.dependencies_of(node)?.to_vec();
        edit(&mut desired);
        self.set_dependencies(node, desired)
    }

    pub fn apply_dependency_batch_edit(
        &mut self,
        edit: &DependencyBatchEdit,
    ) -> Result<(), SignalError> {
        let reconciliations = edit
            .as_slice()
            .iter()
            .map(|entry| (entry.node, entry.dependencies.clone()))
            .collect::<Vec<_>>();
        let _ = self.reconcile_dependencies_batch(&reconciliations)?;
        Ok(())
    }

    pub(crate) fn build_dependency_edge(
        &mut self,
        upstream: NodeId,
        aspect: Aspect,
        scope: Option<PartitionSubscription>,
    ) -> DependencyEdge {
        match scope {
            Some(scope) => {
                let token_count_before = self.observation.partition_interner.token_count();
                let interned_scope = self
                    .observation
                    .partition_interner
                    .intern_subscription(&scope);
                self.observation
                    .telemetry
                    .invalidation
                    .partition_interner_growth_delta +=
                    self.observation
                        .partition_interner
                        .token_count()
                        .saturating_sub(token_count_before) as u64;
                DependencyEdge::with_scope(upstream, aspect, scope, interned_scope)
            }
            None => DependencyEdge::new(upstream, aspect),
        }
    }

    pub(super) fn intern_dependency_edges(
        &mut self,
        desired: CanonicalDependencies,
    ) -> CanonicalDependencies {
        CanonicalDependencies::new(desired.as_slice().iter().map(|edge| {
            self.build_dependency_edge(edge.source(), edge.aspect(), edge.scope_ref().cloned())
        }))
    }
}
