use std::collections::{BTreeMap, BTreeSet};

use crate::data::dependency::{DependencyEdge, DependencySnapshotEntry};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;

use super::super::types::{RewiringDependency, RewiringSummary};

#[derive(Debug, Clone, Default)]
pub(super) struct ExplanationTraversalCost {
    dependency_width: u32,
    snapshot_width: u32,
    source_version_lookups: u32,
    scope_validations: u32,
    runtime_artifact_lookups: u32,
    removed_dependency_width: u32,
}

impl ExplanationTraversalCost {
    pub(crate) fn note_source_version_lookup(&mut self) {
        self.source_version_lookups += 1;
    }

    pub(crate) fn note_scope_validation(&mut self) {
        self.scope_validations += 1;
    }

    pub(crate) fn note_runtime_artifact_lookup(&mut self) {
        self.runtime_artifact_lookups += 1;
    }

    pub(super) fn note_removed_dependency(&mut self) {
        self.removed_dependency_width += 1;
    }

    pub(super) fn validate(&self) {
        debug_assert!(self.source_version_lookups <= self.dependency_width);
        debug_assert!(self.removed_dependency_width <= self.snapshot_width);
    }
}

pub(super) struct ExplanationLineage {
    snapshot_entries: Vec<DependencySnapshotEntry>,
    current_dependencies: Vec<DependencyEdge>,
    snapshot_by_dependency: BTreeMap<(NodeId, usize, Option<PartitionSubscription>), u64>,
    current_dependency_set: BTreeSet<(NodeId, usize, Option<PartitionSubscription>)>,
    rewiring: Option<RewiringSummary>,
    traversal_cost: ExplanationTraversalCost,
}

impl ExplanationLineage {
    pub(super) fn collect(graph: &SignalGraph, node: NodeId) -> Result<Self, SignalError> {
        let snapshot_entries = graph.get_dep_snapshot(node)?.entries().to_vec();
        let current_dependencies = graph.dependencies_of(node)?.to_vec();
        let mut snapshot_by_dependency = BTreeMap::new();
        for snapshot_entry in snapshot_entries.iter().cloned() {
            snapshot_by_dependency.insert(
                (
                    snapshot_entry.source,
                    snapshot_entry.aspect.index(),
                    snapshot_entry.scope.clone(),
                ),
                snapshot_entry.cached_version,
            );
        }
        let current_dependency_set = current_dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.source(),
                    dependency.aspect().index(),
                    dependency.scope_ref().cloned(),
                )
            })
            .collect();
        let traversal_cost = ExplanationTraversalCost {
            dependency_width: current_dependencies.len() as u32,
            snapshot_width: snapshot_entries.len() as u32,
            ..ExplanationTraversalCost::default()
        };
        let rewiring = rewiring_summary(&snapshot_entries, &current_dependencies);
        Ok(Self {
            snapshot_entries,
            current_dependencies,
            snapshot_by_dependency,
            current_dependency_set,
            rewiring,
            traversal_cost,
        })
    }

    pub(super) fn current_dependencies(&self) -> &[DependencyEdge] {
        &self.current_dependencies
    }

    pub(super) fn snapshot_entries(&self) -> &[DependencySnapshotEntry] {
        &self.snapshot_entries
    }

    pub(super) fn cached_version(
        &self,
        source: NodeId,
        aspect_index: usize,
        scope: Option<&PartitionSubscription>,
    ) -> Option<u64> {
        self.snapshot_by_dependency
            .get(&(source, aspect_index, scope.cloned()))
            .copied()
    }

    pub(super) fn contains_current_dependency(
        &self,
        source: NodeId,
        aspect_index: usize,
        scope: Option<&PartitionSubscription>,
    ) -> bool {
        self.current_dependency_set
            .contains(&(source, aspect_index, scope.cloned()))
    }

    pub(super) fn rewiring(&self) -> Option<RewiringSummary> {
        self.rewiring.clone()
    }

    pub(super) fn traversal_cost(&self) -> &ExplanationTraversalCost {
        &self.traversal_cost
    }

    pub(crate) fn traversal_cost_mut(&mut self) -> &mut ExplanationTraversalCost {
        &mut self.traversal_cost
    }
}

fn rewiring_summary(
    snapshot_entries: &[DependencySnapshotEntry],
    current_dependencies: &[DependencyEdge],
) -> Option<RewiringSummary> {
    let current = current_dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.source(),
                dependency.aspect(),
                dependency.scope_ref().cloned(),
            )
        })
        .collect::<Vec<_>>();
    let snapshot = snapshot_entries
        .iter()
        .map(|entry| (entry.source, entry.aspect, entry.scope.clone()))
        .collect::<Vec<_>>();

    let mut added = current
        .iter()
        .filter(|candidate| !snapshot.contains(candidate))
        .map(|(source, aspect, subscription)| RewiringDependency {
            source: *source,
            aspect: *aspect,
            subscription: subscription.clone(),
        })
        .collect::<Vec<_>>();
    let mut removed = snapshot
        .iter()
        .filter(|candidate| !current.contains(candidate))
        .map(|(source, aspect, subscription)| RewiringDependency {
            source: *source,
            aspect: *aspect,
            subscription: subscription.clone(),
        })
        .collect::<Vec<_>>();

    if added.is_empty() && removed.is_empty() {
        None
    } else {
        canonicalize_rewiring_dependencies(&mut added);
        canonicalize_rewiring_dependencies(&mut removed);
        Some(RewiringSummary { added, removed })
    }
}

fn canonicalize_rewiring_dependencies(dependencies: &mut [RewiringDependency]) {
    dependencies.sort_by_key(rewiring_key);
}

fn rewiring_key(dependency: &RewiringDependency) -> (u32, u32, usize, Vec<String>, u8) {
    let scope = dependency.subscription.as_ref().map(|subscription| {
        (
            subscription.path().segments().to_vec(),
            subscription.coverage() as u8,
        )
    });
    (
        dependency.source.index(),
        dependency.source.generation(),
        dependency.aspect.index(),
        scope
            .as_ref()
            .map(|(path, _)| path.clone())
            .unwrap_or_default(),
        scope.as_ref().map(|(_, mode)| *mode).unwrap_or_default(),
    )
}
