use super::*;
use worth_execution::{
    ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial, WorkCeilingDenial,
};

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "original typed refusal is retained opaquely; disclosure uses approved projections"
)]
pub(in crate::domain_computation::primary_graph::application_resource) enum ResourceStorageAllocationDenial
{
    Overflow,
    Request(WorkCeilingDenial),
    Array(ExecutionRequestArrayDenial),
}

impl ResourceStorage {
    pub(in crate::domain_computation::primary_graph::application_resource) fn allocate(
        execution: &ExecutionRequest<'_, '_>,
        nodes: ExecutionArray<NativeResourceSlot>,
        retained_results: usize,
    ) -> Result<Arc<Self>, ResourceStorageAllocationDenial> {
        use ResourceStorageAllocationDenial as Denial;
        let row_count = nodes
            .len()
            .checked_add(retained_results)
            .ok_or(Denial::Overflow)?;
        if nodes.is_empty() || retained_results == 0 {
            return Err(Denial::Overflow);
        }
        let bytes = std::mem::size_of::<Self>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::Overflow)?;
        let funding = (*execution)
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        let mut rows =
            ExecutionRequestArrayBuilder::allocate(*execution, row_count).map_err(Denial::Array)?;
        for index in 0..row_count {
            rows.push(Mutex::new(RowSlot {
                generation: 0,
                next_free: (index + 1 < row_count).then_some(index + 1),
                allocated: false,
                abandoned: false,
                occurrence: None,
            }))
            .map_err(Denial::Array)?;
        }
        let mut results = ExecutionRequestArrayBuilder::allocate(*execution, retained_results)
            .map_err(Denial::Array)?;
        for index in 0..retained_results {
            results
                .push(Mutex::new(ResultSlot {
                    next_free: (index + 1 < retained_results).then_some(index + 1),
                    owner: None,
                }))
                .map_err(Denial::Array)?;
        }
        Ok(Arc::new(Self {
            nodes,
            rows: rows.seal().map_err(Denial::Array)?,
            results: results.seal().map_err(Denial::Array)?,
            free: Mutex::new(FreeSlots {
                row: Some(0),
                native: Some(0),
                result: Some(0),
            }),
            _funding: funding,
        }))
    }

    pub(in crate::domain_computation::primary_graph::application_resource) fn reserve(
        self: &Arc<Self>,
        occurrence: ResourceOccurrence,
    ) -> Result<ResourceRowLease, (ResourceOccurrence, ResourceStorageDenial)> {
        let mut free = lock(&self.free);
        let Some(index) = free.row else {
            return Err((occurrence, ResourceStorageDenial::RowCapacity));
        };
        let mut slot = lock(&self.rows[index]);
        let Some(generation) = slot.generation.checked_add(1) else {
            return Err((occurrence, ResourceStorageDenial::RowCapacity));
        };
        free.row = slot.next_free.take();
        slot.generation = generation;
        slot.allocated = true;
        slot.abandoned = false;
        Ok(ResourceRowLease::new(
            Arc::clone(self),
            ResourceRowKey { index, generation },
            occurrence,
        ))
    }
}

impl NativeResourceSlot {
    pub(in crate::domain_computation::primary_graph::application_resource) fn new(
        declaration: LoweredBridgeAsyncSourceDeclaration,
        ingress: Arc<IngressBacking>,
        next_free: Option<usize>,
    ) -> Self {
        Self {
            declaration,
            ingress,
            next_free: Mutex::new(next_free),
        }
    }
}
