use super::*;
use crate::data::temporal::TemporalClockAdvanceSummary;

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn advance_conditional_owned_async_clock(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
        definition: &SignalInstalledDefinitionBinding,
        node: ResourceNodeId,
        coordinate: u64,
    ) -> Result<TemporalClockAdvanceSummary, SignalConditionalServiceExecutionDenial> {
        self.with_conditional_owned_async_state(admission, basis, definition, |state| {
            state.advance_owned_async_clock(node, coordinate)
        })
    }
}
