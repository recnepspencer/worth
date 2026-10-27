use std::sync::Arc;

use crate::identity::data::PartitionId;
use crate::indexes::data::DerivedIndexId;
use crate::transactions::data::RecordRef;
use crate::validation::data::{InvariantExecutionPoint, InvariantGroupSet};
use crate::validation::engine::InvariantObservationKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ValidationReductionKey {
    pub(crate) execution_point: InvariantExecutionPoint,
    pub(crate) observation_kind: InvariantObservationKind,
    pub(crate) partition_scope: Arc<[PartitionId]>,
    pub(crate) packet_index: usize,
    pub(crate) invariant_group_scope_mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DiffReductionKey {
    pub(crate) target: RecordRef,
    pub(crate) kind_order: u8,
    pub(crate) packet_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct IndexReductionKey(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ImportReductionKey(u64);

impl DiffReductionKey {
    pub(crate) fn new(target: RecordRef, kind_order: u8, packet_index: usize) -> Self {
        Self {
            target,
            kind_order,
            packet_index,
        }
    }
}

impl IndexReductionKey {
    pub(crate) fn new(index_id: DerivedIndexId, packet_index: usize) -> Self {
        Self(pack_u64_pair(index_id.0, packet_index))
    }
}

impl ImportReductionKey {
    pub(crate) fn new(partition_id: PartitionId, kind_order: u8, packet_index: usize) -> Self {
        debug_assert!(
            packet_index <= u32::MAX as usize,
            "packet index must fit into the packed key contract"
        );
        Self(((partition_id.0 as u64) << 32) | ((kind_order as u64) << 24) | (packet_index as u64))
    }
}

impl ValidationReductionKey {
    pub(crate) fn new(
        execution_point: InvariantExecutionPoint,
        observation_kind: InvariantObservationKind,
        partition_scope: Arc<[PartitionId]>,
        invariant_group_scope: InvariantGroupSet,
        packet_index: usize,
    ) -> Self {
        Self {
            execution_point,
            observation_kind,
            partition_scope,
            invariant_group_scope_mask: invariant_group_scope.mask(),
            packet_index,
        }
    }
}

const fn pack_u64_pair(high: u64, low: usize) -> u64 {
    debug_assert!(low <= u32::MAX as usize, "low word must fit into u32");
    (high << 32) | (low as u64)
}
