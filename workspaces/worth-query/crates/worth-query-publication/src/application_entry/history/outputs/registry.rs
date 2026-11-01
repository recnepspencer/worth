//! Fixed-size funded slots; registry guards never enclose authored work.
use super::{retained::*, *};
use std::sync::Arc;

impl<S, P, R> WorthQueryHistoryWithOutputs<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) fn existing_slot(
        &self,
        key: &[u8; 32],
    ) -> Result<Option<Arc<OutputSlot<S, P, R>>>, WorthQueryHistoryOutputDenial> {
        Ok(self
            .slots
            .try_lock()
            .map_err(|_| WorthQueryHistoryOutputDenial::Busy)?
            .get(key)
            .map(Arc::clone))
    }

    pub(super) fn remove_slot(
        &self,
        key: &[u8; 32],
        original: &Arc<OutputSlot<S, P, R>>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        let removed = {
            let mut registry = self
                .slots
                .try_lock()
                .map_err(|_| WorthQueryHistoryOutputDenial::Busy)?;
            if registry
                .get(key)
                .is_some_and(|slot| Arc::ptr_eq(slot, original))
            {
                registry.remove(key)
            } else {
                None
            }
        };
        drop(removed);
        Ok(())
    }

    pub(super) fn slot(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        key: [u8; 32],
    ) -> Result<Arc<OutputSlot<S, P, R>>, WorthQueryHistoryOutputDenial> {
        use WorthQueryHistoryOutputDenial as Denial;
        let execution = phase
            .checked_execution_request_for(self.application.runtime().product_runtime())
            .map_err(Denial::Execution)?;
        let mut registry = self.slots.try_lock().map_err(|_| Denial::Busy)?;
        let levels = u64::from(usize::BITS - registry.len().saturating_add(1).leading_zeros()) + 1;
        execution
            .run_native_with_checkpoint(|context| context.checkpoint(levels * 11))
            .map_err(Denial::Work)?
            .0
            .map_err(|cause| Denial::Work(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
        if let Some(slot) = registry.get(&key) {
            return Ok(Arc::clone(slot));
        }
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<OutputSlot<S, P, R>>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        // One insertion may split each level and grow a root. The same bound
        // covers std BTree's eleven key/value slots and fourteen pointer words.
        let node_bytes = 11_u64
            * std::mem::size_of::<([u8; 32], Arc<OutputSlot<S, P, R>>)>() as u64
            + 14 * std::mem::size_of::<usize>() as u64;
        let bytes = levels
            .checked_mul(node_bytes)
            .and_then(|n| n.checked_add(layout.size() as u64))
            .ok_or(Denial::CapacityOverflow)?;
        let memory = match execution
            .in_scope(|lease| {
                worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
            })
            .map_err(Denial::Work)?
        {
            Ok(reservation) => Some(reservation),
            Err(worth_execution::LeaseDenial::NoActiveExecutionScope)
                if !execution.has_accounting() =>
            {
                None
            }
            Err(cause) => return Err(Denial::Memory(cause)),
        };
        let slot = Arc::new(OutputSlot::new(memory));
        registry.insert(key, Arc::clone(&slot));
        Ok(slot)
    }
}
