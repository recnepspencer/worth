//! Release only the original medium's local physical owner shares.
use super::StoreJournalMedium;
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RelationalDurableColdSource, RelationalMediumRetirementDenial as Denial,
    RelationalMediumRetirementProgress as Progress,
};

impl StoreJournalMedium {
    pub(super) fn preflight_home_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        let original = self.cold_source.as_ref().ok_or(Denial::Unsupported)?;
        if !std::ptr::addr_eq(original.as_ref(), source) {
            return Err(Denial::ForeignSource);
        }
        if !self
            .serving
            .as_ref()
            .is_some_and(|serving| original.belongs_to(serving))
        {
            return Err(Denial::ForeignSource);
        }
        Ok(())
    }
    pub(super) fn advance_home_retirement(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Progress, Denial> {
        self.preflight_home_retirement(source, request)?;
        // The validated borrowed original source owns another Serving share;
        // neither release can destroy an original provider payload here.
        drop(self.serving.take());
        drop(self.cold_source.take());
        Ok(Progress::Complete)
    }
}
