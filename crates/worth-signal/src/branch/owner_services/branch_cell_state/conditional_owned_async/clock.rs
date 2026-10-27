use super::*;
use crate::data::temporal::TemporalClockAdvanceSummary;

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn advance_owned_async_clock(
        &mut self,
        node: ResourceNodeId,
        coordinate: u64,
    ) -> Result<TemporalClockAdvanceSummary, SignalConditionalServiceExecutionDenial> {
        self.state.advance_owned_async_clock(node, coordinate)
    }
}
