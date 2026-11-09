//! Snapshot membership without performing any component call under the lock.
use super::*;
use crate::branch::{
    ObservedProductBranches, ProductBranchInventoryDenial as Denial,
    RuntimeWorldBranchAdmissionDenial,
};
impl ProductBranchRegistry {
    pub(crate) fn observe_installed(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observe: impl Fn(
            ProductBranchReferenceCell,
        ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial>,
    ) -> Result<ObservedProductBranches, Denial> {
        execution.consult().map_err(Denial::Request)?;
        let (cells, memory) = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let bytes = state
                .entries
                .len()
                .checked_mul(
                    std::mem::size_of::<ProductBranchReferenceCell>()
                        + std::mem::size_of::<ProductBranchObservation>(),
                )
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Denial::AllocationRefused)?;
            let memory = execution
                .reserve_retained_memory(bytes)
                .map_err(Denial::Request)?;
            let mut cells = Vec::new();
            cells
                .try_reserve_exact(state.entries.len())
                .map_err(|_| Denial::AllocationRefused)?;
            for key in state.entries.keys() {
                execution.consult().map_err(Denial::Request)?;
                cells.push(
                    state
                        .entries
                        .get(key)
                        .expect("same installed membership")
                        .cell
                        .clone(),
                );
            }
            (cells, memory)
        };
        let mut branches = Vec::new();
        branches
            .try_reserve_exact(cells.len())
            .map_err(|_| Denial::AllocationRefused)?;
        for cell in cells {
            execution.consult().map_err(Denial::Request)?;
            branches.push(observe(cell).map_err(Denial::Branch)?);
        }
        Ok(ObservedProductBranches {
            branches,
            _memory: memory,
        })
    }
}
