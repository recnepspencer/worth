use std::sync::Arc;

use serde::{Deserialize, Serialize};

mod retained_charge;

use crate::data::aspect::{Aspect, AspectMask};
use crate::data::handle::NodeId;
use crate::data::output::{InternedPartitionSubscription, PartitionSubscription, PartitionToken};

/// A dependency edge recording which upstream node and aspect a downstream reads.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DependencyEdge {
    source: NodeId,
    aspect: Aspect,
    #[serde(default)]
    scope: Option<PartitionSubscription>,
    #[serde(default)]
    interned_scope: Option<InternedPartitionSubscription>,
}

impl DependencyEdge {
    /// Create a new dependency edge.
    pub fn new(source: NodeId, aspect: Aspect) -> Self {
        Self {
            source,
            aspect,
            scope: None,
            interned_scope: None,
        }
    }

    /// Create a partition-scoped dependency edge.
    pub fn with_scope(
        source: NodeId,
        aspect: Aspect,
        scope: PartitionSubscription,
        interned_scope: InternedPartitionSubscription,
    ) -> Self {
        Self {
            source,
            aspect,
            scope: Some(scope),
            interned_scope: Some(interned_scope),
        }
    }

    /// Create a dependency edge scoped to one whole partition.
    pub fn whole_partition(
        source: NodeId,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
    ) -> Self {
        Self {
            source,
            aspect,
            scope: Some(PartitionSubscription::whole_partition(partition)),
            interned_scope: None,
        }
    }

    /// Create a dependency edge from an explicit partition subscription scope.
    pub fn with_partition_scope(
        source: NodeId,
        aspect: Aspect,
        scope: PartitionSubscription,
    ) -> Self {
        Self {
            source,
            aspect,
            scope: Some(scope),
            interned_scope: None,
        }
    }

    /// Create a dependency edge scoped to one partition detail.
    pub fn partition_detail(
        source: NodeId,
        aspect: Aspect,
        partition: impl Into<PartitionToken>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            source,
            aspect,
            scope: Some(PartitionSubscription::partition_and_detail(
                partition, detail,
            )),
            interned_scope: None,
        }
    }

    /// The upstream node this edge points to.
    pub fn source(&self) -> NodeId {
        self.source
    }

    /// Which aspect of the upstream node is subscribed to.
    pub fn aspect(&self) -> Aspect {
        self.aspect
    }

    /// Optional partition subscription scope.
    pub fn scope_ref(&self) -> Option<&PartitionSubscription> {
        self.scope.as_ref()
    }

    /// Optional compact interned scope for hot-path routing.
    pub fn interned_scope(&self) -> Option<InternedPartitionSubscription> {
        self.interned_scope
    }

    /// The subscribed aspect mask.
    pub fn aspect_mask(&self) -> AspectMask {
        AspectMask::from_aspect(self.aspect)
    }

    pub fn sort_key(&self) -> DependencySortKey {
        DependencySortKey {
            source_index: self.source.index(),
            source_generation: self.source.generation(),
            aspect_index: self.aspect.index(),
            scope: self.scope.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DependencySortKey {
    pub(super) source_index: u32,
    pub(super) source_generation: u32,
    pub(super) aspect_index: usize,
    pub(super) scope: Option<PartitionSubscription>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CanonicalDependencies {
    edges: Arc<Vec<DependencyEdge>>,
}

impl CanonicalDependencies {
    pub fn new(edges: impl IntoIterator<Item = DependencyEdge>) -> Self {
        Self::canonicalize_unordered(edges)
    }

    pub fn canonicalize_unordered(edges: impl IntoIterator<Item = DependencyEdge>) -> Self {
        let mut edges = edges.into_iter().collect::<Vec<_>>();
        if edges.len() > 1 {
            edges.sort_by_key(|left| left.sort_key());
            edges.dedup_by(|left, right| left.sort_key() == right.sort_key());
        }
        Self {
            edges: Arc::new(edges),
        }
    }

    pub fn from_ordered_unique(edges: impl IntoIterator<Item = DependencyEdge>) -> Self {
        let edges = edges.into_iter().collect::<Vec<_>>();
        debug_assert!(is_strict_dependency_edge_order(edges.as_slice()));
        Self {
            edges: Arc::new(edges),
        }
    }

    pub fn from_slice(edges: &[DependencyEdge]) -> Self {
        Self::new(edges.iter().cloned())
    }

    pub fn as_slice(&self) -> &[DependencyEdge] {
        self.edges.as_slice()
    }

    pub fn into_vec(self) -> Vec<DependencyEdge> {
        match Arc::try_unwrap(self.edges) {
            Ok(edges) => edges,
            Err(edges) => edges.as_ref().clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}

impl From<Vec<DependencyEdge>> for CanonicalDependencies {
    fn from(edges: Vec<DependencyEdge>) -> Self {
        Self::new(edges)
    }
}

impl From<&[DependencyEdge]> for CanonicalDependencies {
    fn from(edges: &[DependencyEdge]) -> Self {
        Self::from_slice(edges)
    }
}

fn is_strict_dependency_edge_order(edges: &[DependencyEdge]) -> bool {
    edges
        .windows(2)
        .all(|pair| pair[0].sort_key() < pair[1].sort_key())
}
