use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::identity::data::PartitionId;

use super::root::{
    RelationalBranchRootCaptureDenial, RelationalBranchRootIdentityIssuer, RelationalRootRegion,
};

pub mod source_retirement;
pub use source_retirement::{RelationalRegionSourceWalk, RelationalRegionSourceWalkProgress};
pub mod insertion;
use insertion::insert_leaf;
pub mod request_construction;
pub(in crate::internal_port::branch) use request_construction::PreparedRelationalRegionSet;
pub mod allocation_visitation;
pub mod commitment;
pub mod publication;
use commitment::{empty_commitment, node_commitment};
pub mod partition_visitation;
pub use partition_visitation::RelationalPartitionVisit;

pub mod backing;
pub use backing::RelationalPersistentRegionSet;
use backing::{
    get_region, key_bit, RelationalPersistentRegionLeaf, RelationalPersistentRegionNode,
    RelationalRemovedPartition, RelationalRootRegionReference, PARTITION_KEY_BITS,
};
pub mod selected_access;

impl RelationalPersistentRegionSet {
    pub fn materialize(&self) -> BTreeMap<PartitionId, Arc<RelationalRootRegion>> {
        let mut regions = BTreeMap::new();
        materialize_node(self.index_root.as_ref(), 0, 0, &mut regions);
        regions
    }

    pub fn values(&self) -> impl Iterator<Item = Arc<RelationalRootRegion>> + use<> {
        self.materialize().into_values()
    }
    pub fn commitment(&self) -> [u8; 32] {
        self.index_root
            .as_ref()
            .map_or_else(empty_commitment, |root| root.commitment)
    }

    pub fn allocation_observations(&self) -> Vec<RelationalPersistentRegionNodeObservation> {
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
pub enum RelationalPersistentRegionAllocationKind {
    SetObject,
    MapNodeObject,
    ReplacementStorage,
    RemovalStorage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalPersistentRegionNodeObservation {
    pub node_id: u64,
    pub allocation_kind: RelationalPersistentRegionAllocationKind,
    pub authoritative_bytes: u64,
}

#[cfg(test)]
#[path = "root_regions_tests.rs"]
mod tests;

#[path = "root_regions_reclamation_accounting.rs"]
pub mod reclamation_accounting;
