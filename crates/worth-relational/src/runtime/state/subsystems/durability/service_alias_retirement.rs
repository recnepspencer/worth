//! Release proven local shares while the same original binding stays installed.
use super::*;
use crate::durability::medium::RelationalMediumRetirementDenial as Denial;
use worth_execution::ExecutionRequest;
impl DurabilitySubsystem {
    pub(crate) fn retire_local_service_medium_aliases(
        &self,
        request: &ExecutionRequest<'_, '_>,
        fork: &mut Option<Arc<RelationalDurableMediumBinding>>,
        publication: &mut Option<Arc<RelationalDurableMediumBinding>>,
    ) -> Result<(), Denial> {
        if fork.is_none() && publication.is_none() {
            return Ok(());
        }
        let state = self.state.try_read().ok_or(Denial::Busy)?;
        let actual = state.medium.as_ref().ok_or(Denial::MediumMoved)?;
        for alias in [fork.as_ref(), publication.as_ref()].into_iter().flatten() {
            if !Arc::ptr_eq(actual, alias) {
                return Err(Denial::ForeignSource);
            }
        }
        request.consult().map_err(Denial::Execution)?;
        // The guarded owner slot still owns this EXACT binding. Only nonfinal
        // reference counters change; no medium destructor or callback can run.
        drop(fork.take());
        drop(publication.take());
        Ok(())
    }
}
