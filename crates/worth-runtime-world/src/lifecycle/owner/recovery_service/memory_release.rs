//! Release keeps exact Native original data after retiring live Memory custody.
use super::RuntimeWorldOwnerRoot;
use crate::recovery::{
    ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle,
    RuntimeWorldRecoveryDenial as Denial, WorldReleasedEffects,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::ObservedRelationalMemoryCommit;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn release_memory_original(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        handle: &ProductUnpublishedRecoveryHandle,
        minimum_age_ticks: u64,
        native: Option<ObservedRelationalMemoryCommit>,
    ) -> Result<WorldReleasedEffects, Denial> {
        if self.state.durable.is_some() {
            return Err(Denial::ReopenRequired);
        }
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| super::super::super::RuntimeWorldOwnerUnavailable::new())?;
        let record = self.state.recovery.inspect_record(handle)?;
        let subject = std::sync::Arc::downgrade(&record);
        let effects = ProductUnpublishedOwnerEffects::from_catalog_record(record.clone());
        match (
            effects
                .component_results()
                .release_original_component()
                .map_err(|()| Denial::SettlementRequired)?,
            native.as_ref(),
        ) {
            (Some((receipt, basis)), Some(native)) => {
                let component = self
                    .state
                    .relational_services()
                    .basis_port()
                    .observe_memory_component(execution, basis)
                    .map_err(|_| Denial::OriginalSourceUnavailable)?;
                if receipt != native.receipt() || !component.matches_original(native) {
                    return Err(Denial::OriginalSourceMismatch);
                }
            }
            (None, None) => {}
            (Some(_), None) => return Err(Denial::OriginalSourceUnavailable),
            (None, Some(_)) => return Err(Denial::OriginalSourceMismatch),
        }
        let destination = self
            .state
            .recovery
            .prepare_memory_original_release(execution, &record)
            .map_err(|_| Denial::OutputCapacityExhausted)?;
        drop((effects, record));
        let work = self.reserve_recovery_retirement_work(handle)?;
        let mut candidates = Vec::new();
        candidates
            .try_reserve_exact(1)
            .map_err(|_| Denial::OutputCapacityExhausted)?;
        let mut original = None;
        let released = self.state.recovery.cleanup_record(
            handle,
            Some((self.state.clock.now(), minimum_age_ticks)),
            Some(&subject),
            || {
                original = Some(destination.finish_memory(native));
            },
        )?;
        let original = original.expect("successful exclusive removal installed its source");
        if let Some(commit) = released.unpublished_commit() {
            candidates.push(commit.clone());
        }
        let retirement = self.drain_released_occurrence(&released, work);
        Ok(WorldReleasedEffects::new(
            original,
            crate::recovery::ProductUnpublishedCleanup::new(candidates, retirement),
        ))
    }
}
