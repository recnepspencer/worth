use std::sync::Arc;

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::data::handle::NodeId;
use crate::data::output::{CanonicalChangedRegions, PartitionSubscription};

use super::{CanonicalForm, SummaryForm};

mod retained_charge;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DedupedNodeBatch {
    nodes: Arc<Vec<NodeId>>,
}

impl DedupedNodeBatch {
    pub fn new(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        Self::canonicalize_unordered(nodes)
    }

    pub fn canonicalize_unordered(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        let mut nodes = nodes.into_iter().collect::<Vec<_>>();
        if nodes.len() > 1 {
            nodes.sort_unstable_by_key(node_sort_key);
            nodes.dedup();
        }
        Self {
            nodes: Arc::new(nodes),
        }
    }

    pub fn from_ordered_unique(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        let nodes = nodes.into_iter().collect::<Vec<_>>();
        debug_assert!(is_strict_node_order(nodes.as_slice()));
        Self {
            nodes: Arc::new(nodes),
        }
    }

    pub fn from_slice(nodes: &[NodeId]) -> Self {
        Self::new(nodes.iter().copied())
    }

    pub fn as_slice(&self) -> &[NodeId] {
        self.nodes.as_slice()
    }

    pub fn into_vec(self) -> Vec<NodeId> {
        match Arc::try_unwrap(self.nodes) {
            Ok(nodes) => nodes,
            Err(nodes) => nodes.as_ref().clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SortedSourceBatch {
    sources: Arc<Vec<NodeId>>,
}

impl SortedSourceBatch {
    pub fn new(sources: impl IntoIterator<Item = NodeId>) -> Self {
        Self::canonicalize_unordered(sources)
    }

    pub fn canonicalize_unordered(sources: impl IntoIterator<Item = NodeId>) -> Self {
        let mut sources = sources.into_iter().collect::<Vec<_>>();
        if sources.len() > 1 {
            sources.sort_unstable_by_key(node_sort_key);
            sources.dedup();
        }
        Self {
            sources: Arc::new(sources),
        }
    }

    pub fn from_ordered_unique(sources: impl IntoIterator<Item = NodeId>) -> Self {
        let sources = sources.into_iter().collect::<Vec<_>>();
        debug_assert!(is_strict_node_order(sources.as_slice()));
        Self {
            sources: Arc::new(sources),
        }
    }

    pub fn from_slice(sources: &[NodeId]) -> Self {
        Self::new(sources.iter().copied())
    }

    pub fn as_slice(&self) -> &[NodeId] {
        self.sources.as_slice()
    }

    pub fn into_vec(self) -> Vec<NodeId> {
        match Arc::try_unwrap(self.sources) {
            Ok(sources) => sources,
            Err(sources) => sources.as_ref().clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PartitionScopeSet(SmallVec<[PartitionSubscription; 8]>);

impl PartitionScopeSet {
    /// Accept a canonical sequence without re-sorting it. Validate at the
    /// value owner; callers cannot use an unchecked canonicality assertion.
    pub(crate) fn from_canonical_scopes(scopes: Vec<PartitionSubscription>) -> Option<Self> {
        scopes
            .windows(2)
            .all(|pair| pair[0] < pair[1])
            .then(|| Self(SmallVec::from_vec(scopes)))
    }

    pub fn new(scopes: impl IntoIterator<Item = PartitionSubscription>) -> Self {
        let mut scopes = SmallVec::<[PartitionSubscription; 8]>::from_iter(scopes);
        if scopes.len() > 1 {
            scopes.sort_unstable();
            scopes.dedup();
        }
        Self(scopes)
    }

    pub fn from_changed_regions(changed_regions: &CanonicalChangedRegions) -> Self {
        Self::new(
            changed_regions
                .as_slice()
                .iter()
                .map(|region| match &region.detail {
                    Some(detail) => PartitionSubscription::partition_and_detail(
                        region.partition.clone(),
                        detail.clone(),
                    ),
                    None => PartitionSubscription::whole_partition(region.partition.clone()),
                }),
        )
    }

    pub fn as_slice(&self) -> &[PartitionSubscription] {
        self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &PartitionSubscription> {
        self.0.iter()
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let mut left = 0usize;
        let mut right = 0usize;
        while left < self.0.len() && right < other.0.len() {
            match self.0[left].cmp(&other.0[right]) {
                std::cmp::Ordering::Less => left += 1,
                std::cmp::Ordering::Greater => right += 1,
                std::cmp::Ordering::Equal => return true,
            }
        }
        false
    }
}

impl From<Vec<PartitionSubscription>> for PartitionScopeSet {
    fn from(scopes: Vec<PartitionSubscription>) -> Self {
        Self::new(scopes)
    }
}

impl From<&[PartitionSubscription]> for PartitionScopeSet {
    fn from(scopes: &[PartitionSubscription]) -> Self {
        Self::new(scopes.iter().cloned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LocalityFootprint {
    pub partitions: PartitionScopeSet,
    pub nodes: DedupedNodeBatch,
    pub sources: SortedSourceBatch,
}

impl LocalityFootprint {
    pub fn new(
        partitions: impl Into<PartitionScopeSet>,
        nodes: impl Into<DedupedNodeBatch>,
        sources: impl Into<SortedSourceBatch>,
    ) -> Self {
        Self {
            partitions: partitions.into(),
            nodes: nodes.into(),
            sources: sources.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.partitions.is_empty() && self.nodes.is_empty() && self.sources.is_empty()
    }

    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.nodes
            .as_slice()
            .iter()
            .any(|node| other.nodes.as_slice().contains(node))
            || self
                .sources
                .as_slice()
                .iter()
                .any(|node| other.sources.as_slice().contains(node))
    }

    pub fn merge(&mut self, other: &Self) {
        let mut partitions = self.partitions.as_slice().to_vec();
        partitions.extend_from_slice(other.partitions.as_slice());
        self.partitions = partitions.into();

        let mut nodes = self.nodes.as_slice().to_vec();
        nodes.extend_from_slice(other.nodes.as_slice());
        self.nodes = nodes.into();

        let mut sources = self.sources.as_slice().to_vec();
        sources.extend_from_slice(other.sources.as_slice());
        self.sources = sources.into();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TouchedScopeSummary {
    pub seed_scopes: PartitionScopeSet,
    pub inclusion_scopes: PartitionScopeSet,
    pub direct_dirty_scopes: PartitionScopeSet,
    pub maybe_stale_scopes: PartitionScopeSet,
    pub touched_nodes: DedupedNodeBatch,
    pub touched_sources: SortedSourceBatch,
}

impl TouchedScopeSummary {
    pub fn new(
        scopes: impl Into<PartitionScopeSet>,
        touched_nodes: impl Into<DedupedNodeBatch>,
        touched_sources: impl Into<SortedSourceBatch>,
    ) -> Self {
        let scopes = scopes.into();
        Self {
            seed_scopes: scopes.clone(),
            inclusion_scopes: scopes.clone(),
            direct_dirty_scopes: scopes,
            maybe_stale_scopes: PartitionScopeSet::default(),
            touched_nodes: touched_nodes.into(),
            touched_sources: touched_sources.into(),
        }
    }

    pub fn new_invalidation(
        seed_scopes: impl Into<PartitionScopeSet>,
        inclusion_scopes: impl Into<PartitionScopeSet>,
        direct_dirty_scopes: impl Into<PartitionScopeSet>,
        maybe_stale_scopes: impl Into<PartitionScopeSet>,
        touched_nodes: impl Into<DedupedNodeBatch>,
        touched_sources: impl Into<SortedSourceBatch>,
    ) -> Self {
        Self {
            seed_scopes: seed_scopes.into(),
            inclusion_scopes: inclusion_scopes.into(),
            direct_dirty_scopes: direct_dirty_scopes.into(),
            maybe_stale_scopes: maybe_stale_scopes.into(),
            touched_nodes: touched_nodes.into(),
            touched_sources: touched_sources.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.seed_scopes.is_empty()
            && self.inclusion_scopes.is_empty()
            && self.direct_dirty_scopes.is_empty()
            && self.maybe_stale_scopes.is_empty()
            && self.touched_nodes.is_empty()
            && self.touched_sources.is_empty()
    }
}

impl CanonicalForm for DedupedNodeBatch {}
impl CanonicalForm for SortedSourceBatch {}
impl CanonicalForm for PartitionScopeSet {}
impl SummaryForm for LocalityFootprint {}
impl SummaryForm for TouchedScopeSummary {}

pub(super) fn node_sort_key(node: &NodeId) -> (u32, u32) {
    (node.index(), node.generation())
}

fn is_strict_node_order(nodes: &[NodeId]) -> bool {
    nodes
        .windows(2)
        .all(|pair| node_sort_key(&pair[0]) < node_sort_key(&pair[1]))
}
