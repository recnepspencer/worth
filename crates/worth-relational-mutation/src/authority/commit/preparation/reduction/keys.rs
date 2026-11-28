use std::sync::Arc;

use crate::contracts::identity::data::primitives::PartitionId;
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::contracts::validation::data::execution::InvariantExecutionPoint;
use crate::contracts::validation::data::groups::InvariantGroupSet;
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
pub struct DiffReductionKey {
    pub(crate) target: RecordRef,
    pub(crate) kind_order: u8,
    pub(crate) packet_index: usize,
}

impl DiffReductionKey {
    pub fn new(target: RecordRef, kind_order: u8, packet_index: usize) -> Self {
        Self {
            target,
            kind_order,
            packet_index,
        }
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
