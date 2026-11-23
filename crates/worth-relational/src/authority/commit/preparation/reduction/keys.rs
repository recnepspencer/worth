use std::sync::Arc;

use crate::identity::data::PartitionId;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::data::execution::InvariantExecutionPoint;
use crate::validation::data::groups::InvariantGroupSet;
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

impl ImportReductionKey {
    pub(crate) fn new(partition_id: PartitionId, kind_order: u8, packet_index: usize) -> Self {
        debug_assert!(
            /* Proof: PackedReductionKey is private: keys are emitted from admitted packet indices and u32 local ordinals; the low word is masked with u32::MAX before extraction. */
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
