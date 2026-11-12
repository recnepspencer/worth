use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::identity::data::PartitionId;

use super::root::{
    RelationalBranchRootCaptureDenial, RelationalBranchRootIdentityIssuer, RelationalRootRegion,
};

mod source_retirement;
pub(crate) use source_retirement::{
    RelationalRegionSourceWalk, RelationalRegionSourceWalkProgress,
};
mod insertion;
use insertion::insert_leaf;
mod request_construction;
pub(in crate::branch) use request_construction::PreparedRelationalRegionSet;
mod allocation_visitation;
mod commitment;
mod publication;
use commitment::{empty_commitment, node_commitment};
mod partition_visitation;
pub(crate) use partition_visitation::RelationalPartitionVisit;

const PARTITION_KEY_BITS: u32 = u32::BITS;

/// Exact immutable radix index whose fixed 32-bit paths keep ordinary reads
/// and replacements independent of publication ancestry.
#[derive(Debug)]
pub(crate) struct RelationalPersistentRegionSet {
    set_id: u64,
    index_root: Option<Arc<RelationalPersistentRegionNode>>,
    count: usize,
    live_counts: crate::storage::residency::LiveRecordCounts,
    pub(super) kinds: crate::storage::residency::KindRegionDirectory,
    materialization_unavailable_records: usize,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
struct RelationalPersistentRegionNode {
    node_id: u64,
    zero: Option<Arc<Self>>,
    one: Option<Arc<Self>>,
    leaf: Option<RelationalPersistentRegionLeaf>,
    commitment: [u8; 32],
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
enum RelationalPersistentRegionLeaf {
    Present(Box<RelationalRootRegionReference>),
    Removed(Box<RelationalRemovedPartition>),
}
#[derive(Debug)]
struct RelationalRootRegionReference {
    region: Arc<RelationalRootRegion>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
struct RelationalRemovedPartition {
    partition_id: PartitionId,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalPersistentRegionSet {
    pub(crate) fn materialize(&self) -> BTreeMap<PartitionId, Arc<RelationalRootRegion>> {
        let mut regions = BTreeMap::new();
        materialize_node(self.index_root.as_ref(), 0, 0, &mut regions);
        regions
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = Arc<RelationalRootRegion>> + use<> {
        self.materialize().into_values()
    }
    pub(crate) fn get(&self, partition_id: PartitionId) -> Option<&Arc<RelationalRootRegion>> {
        get_region(self.index_root.as_ref(), partition_id)
    }

    pub(crate) fn live_counts(&self) -> crate::storage::residency::LiveRecordCounts {
        self.live_counts
    }
    pub(crate) fn len(&self) -> usize {
        self.count
    }
    pub(crate) fn materialization_unavailable_records(&self) -> usize {
        self.materialization_unavailable_records
    }
    pub(crate) fn commitment(&self) -> [u8; 32] {
        self.index_root
            .as_ref()
            .map_or_else(empty_commitment, |root| root.commitment)
    }

    pub(crate) fn allocation_observations(&self) -> Vec<RelationalPersistentRegionNodeObservation> {
        let mut observations = vec![RelationalPersistentRegionNodeObservation {
            node_id: self.set_id,
            allocation_kind: RelationalPersistentRegionAllocationKind::SetObject,
            authoritative_bytes: std::mem::size_of::<Self>() as u64,
        }];
        observe_node_allocations(self.index_root.as_ref(), &mut observations);
        observations
    }

    pub(super) fn newly_owned_allocation_cost(&self, first_new_node_id: u64) -> (u64, u64) {
        let mut path_nodes = 0_u64;
        let mut authoritative_bytes = std::mem::size_of::<Self>() as u64;
        observe_new_node_cost(
            self.index_root.as_ref(),
            first_new_node_id,
            &mut path_nodes,
            &mut authoritative_bytes,
        );
        (path_nodes, authoritative_bytes)
    }
}

fn observe_new_node_cost(
    current: Option<&Arc<RelationalPersistentRegionNode>>,
    first_new_node_id: u64,
    path_nodes: &mut u64,
    authoritative_bytes: &mut u64,
) {
    let Some(node) = current else { return };
    if node.node_id < first_new_node_id {
        return;
    }
    *path_nodes = path_nodes.saturating_add(1);
    *authoritative_bytes = authoritative_bytes
        .saturating_add(std::mem::size_of::<RelationalPersistentRegionNode>() as u64);
    if let Some(leaf) = node.leaf.as_ref() {
        *authoritative_bytes = authoritative_bytes.saturating_add(match leaf {
            RelationalPersistentRegionLeaf::Present(value) => {
                std::mem::size_of_val(value.as_ref()) as u64
            }
            RelationalPersistentRegionLeaf::Removed(value) => {
                std::mem::size_of_val(value.as_ref()) as u64
            }
        });
    }
    observe_new_node_cost(
        node.zero.as_ref(),
        first_new_node_id,
        path_nodes,
        authoritative_bytes,
    );
    observe_new_node_cost(
        node.one.as_ref(),
        first_new_node_id,
        path_nodes,
        authoritative_bytes,
    );
}

fn get_region(
    mut current: Option<&Arc<RelationalPersistentRegionNode>>,
    partition_id: PartitionId,
) -> Option<&Arc<RelationalRootRegion>> {
    for depth in 0..PARTITION_KEY_BITS {
        let node = current?;
        current = if key_bit(partition_id, depth) == 0 {
            node.zero.as_ref()
        } else {
            node.one.as_ref()
        };
    }
    match current?.leaf.as_ref()? {
        RelationalPersistentRegionLeaf::Present(region) => Some(&region.region),
        RelationalPersistentRegionLeaf::Removed(_) => None,
    }
}

fn key_bit(partition_id: PartitionId, depth: u32) -> u32 {
    (partition_id.0 >> (PARTITION_KEY_BITS - depth - 1)) & 1
}

fn materialize_node(
    current: Option<&Arc<RelationalPersistentRegionNode>>,
    depth: u32,
    key_prefix: u32,
    regions: &mut BTreeMap<PartitionId, Arc<RelationalRootRegion>>,
) {
    let Some(node) = current else { return };
    if depth == PARTITION_KEY_BITS {
        if let Some(RelationalPersistentRegionLeaf::Present(region)) = node.leaf.as_ref() {
            regions.insert(PartitionId(key_prefix), Arc::clone(&region.region));
        }
        return;
    }
    materialize_node(node.zero.as_ref(), depth + 1, key_prefix << 1, regions);
    materialize_node(node.one.as_ref(), depth + 1, (key_prefix << 1) | 1, regions);
}

fn observe_node_allocations(
    current: Option<&Arc<RelationalPersistentRegionNode>>,
    observations: &mut Vec<RelationalPersistentRegionNodeObservation>,
) {
    let Some(node) = current else { return };
    observations.push(RelationalPersistentRegionNodeObservation {
        node_id: node.node_id,
        allocation_kind: RelationalPersistentRegionAllocationKind::MapNodeObject,
        authoritative_bytes: std::mem::size_of::<RelationalPersistentRegionNode>() as u64,
    });
    if let Some(leaf) = node.leaf.as_ref() {
        let (allocation_kind, authoritative_bytes) = match leaf {
            RelationalPersistentRegionLeaf::Present(value) => (
                RelationalPersistentRegionAllocationKind::ReplacementStorage,
                std::mem::size_of_val(value.as_ref()) as u64,
            ),
            RelationalPersistentRegionLeaf::Removed(value) => (
                RelationalPersistentRegionAllocationKind::RemovalStorage,
                std::mem::size_of_val(value.as_ref()) as u64,
            ),
        };
        observations.push(RelationalPersistentRegionNodeObservation {
            node_id: node.node_id,
            allocation_kind,
            authoritative_bytes,
        });
    }
    observe_node_allocations(node.zero.as_ref(), observations);
    observe_node_allocations(node.one.as_ref(), observations);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalPersistentRegionAllocationKind {
    SetObject,
    MapNodeObject,
    ReplacementStorage,
    RemovalStorage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelationalPersistentRegionNodeObservation {
    pub(crate) node_id: u64,
    pub(crate) allocation_kind: RelationalPersistentRegionAllocationKind,
    pub(crate) authoritative_bytes: u64,
}

#[cfg(test)]
#[path = "root_regions_tests.rs"]
mod tests;

#[path = "root_regions_reclamation_accounting.rs"]
mod reclamation_accounting;
