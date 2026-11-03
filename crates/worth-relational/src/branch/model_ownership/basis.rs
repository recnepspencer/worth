//! Structural ownership at an admitted immutable branch basis; never today's root.
use super::{traversal::Read, *};
use crate::{
    branch::AdmittedRelationalBranchBasis, runtime::RelationalRuntime,
    storage::overlay::CurrentPartitionAccess, validation::engine::state_view::InvariantStateView,
};
use worth_execution::{ExecutionRequest, MapKernelContext};

impl RelationalRuntime {
    pub fn observe_model_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        subject: EntityId,
    ) -> Result<Option<EntityId>, RelationalModelOwnershipDenial> {
        let (owner, _memory) = observe(self, execution, basis, |read| {
            ownership.owner(read, subject)
        })?;
        Ok(owner)
    }
}

/// Keep result backing alongside any traversal material that escapes observation.
pub(super) fn observe<R>(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    operation: impl FnOnce(&mut dyn Read) -> Result<R, RelationalModelOwnershipDenial>,
) -> Result<(R, Vec<worth_execution::ExecutionMemoryReservation>), RelationalModelOwnershipDenial> {
    let mut result = None;
    execution
        .run_native_with_checkpoint(|context| {
            result = Some(read(runtime, execution, basis, context, operation));
        })
        .map_err(RelationalModelOwnershipDenial::Request)?;
    result.expect("completed native observation")
}

/// A dependent read uses its caller's original checkpoint, never a fresh meter.
pub(super) fn read<R>(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    context: &mut MapKernelContext<'_, '_>,
    operation: impl FnOnce(&mut dyn Read) -> Result<R, RelationalModelOwnershipDenial>,
) -> Result<(R, Vec<worth_execution::ExecutionMemoryReservation>), RelationalModelOwnershipDenial> {
    if basis.identity().runtime_instance_id() != runtime.runtime_instance_id() {
        return Err(RelationalModelOwnershipDenial::ForeignBasis);
    }
    let version = basis
        .inner
        .root
        .axes()
        .ok_or(RelationalModelOwnershipDenial::ForeignBasis)?
        .storage_version;
    let mut read = BasisRead {
        execution,
        context,
        basis,
        memory: Vec::new(),
        state: InvariantStateView::new(
            basis.inner.root.as_ref(),
            crate::identity::data::VersionId(version),
        ),
    };
    operation(&mut read).map(|value| (value, read.memory))
}

struct BasisRead<'a, 'basis, 'context, 'kernel, 'execution, 'request> {
    execution: &'a ExecutionRequest<'execution, 'request>,
    context: &'a mut MapKernelContext<'context, 'kernel>,
    state: InvariantStateView<'basis>,
    basis: &'basis AdmittedRelationalBranchBasis,
    memory: Vec<worth_execution::ExecutionMemoryReservation>,
}

impl BasisRead<'_, '_, '_, '_, '_, '_> {
    fn charge(&mut self, units: usize) -> Result<(), RelationalModelOwnershipDenial> {
        self.context.checkpoint(units as u64).map_err(|stop| {
            RelationalModelOwnershipDenial::Request(worth_execution::WorkCeilingDenial::Stopped(
                stop,
            ))
        })
    }
    fn demand(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Result<(), RelationalModelOwnershipDenial> {
        self.basis
            .inner
            .root
            .demand_head_partition(self.execution, id)
            .map_err(|cause| RelationalModelOwnershipDenial::Native(cause.into()))
    }
}

impl Read for BasisRead<'_, '_, '_, '_, '_, '_> {
    fn kind(&mut self, entity: EntityId) -> Result<Option<KindId>, RelationalModelOwnershipDenial> {
        self.charge(1)?;
        self.demand(entity.partition_id)?;
        Ok(self
            .state
            .entity_metadata(entity)
            .map(|metadata| metadata.kind_id))
    }
    fn adjacent(
        &mut self,
        entity: EntityId,
        outgoing: bool,
        relations: &[KindId],
    ) -> Result<Vec<EntityId>, RelationalModelOwnershipDenial> {
        self.demand(entity.partition_id)?;
        let count = self.state.relation_candidate_count(entity, outgoing);
        self.retain_bytes(count.saturating_mul(512) as u64)?;
        self.charge(
            count
                .saturating_mul(3)
                .saturating_add(relations.len())
                .saturating_add(1),
        )?;
        let ids = self.state.relation_candidates(entity, outgoing);
        let mut neighbors = Vec::new();
        for &id in ids.iter() {
            self.demand(id.partition_id)?;
            if let Some(edge) = self.state.relation_metadata(id) {
                self.charge(relations.len())?;
                if relations.contains(&edge.kind_id)
                    && if outgoing {
                        edge.source == entity
                    } else {
                        edge.target == entity
                    }
                {
                    self.retain()?;
                    neighbors.push(if outgoing { edge.target } else { edge.source });
                }
            }
        }
        Ok(neighbors)
    }
    // Cover a first BTree node, growing queues and reservation-handle backing.
    fn retain(&mut self) -> Result<(), RelationalModelOwnershipDenial> {
        self.charge(1)?;
        self.retain_bytes(512)
    }
}
impl BasisRead<'_, '_, '_, '_, '_, '_> {
    fn retain_bytes(&mut self, bytes: u64) -> Result<(), RelationalModelOwnershipDenial> {
        if let Some(memory) = self
            .execution
            .reserve_retained_memory(bytes)
            .map_err(RelationalModelOwnershipDenial::Request)?
        {
            self.memory.push(memory);
        }
        Ok(())
    }
}
