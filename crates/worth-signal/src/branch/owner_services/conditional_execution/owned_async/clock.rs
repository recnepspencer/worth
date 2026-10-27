//! Clock movement at the exact admitted resource cell, not another partition.
use super::*;
use crate::data::temporal::TemporalClockAdvanceSummary;

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn advance_owned_async_clock(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        coordinate: u64,
    ) -> Result<TemporalClockAdvanceSummary, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.advance_conditional_owned_async_clock(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                coordinate,
            )
        })
    }
}
