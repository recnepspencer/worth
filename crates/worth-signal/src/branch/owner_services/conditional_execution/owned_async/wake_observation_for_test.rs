use super::*;
use crate::data::temporal::{TemporalFrontierSnapshot, TemporalWakeSummary};

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Read-only certification observation of this source's actual branch cell.
    /// It cannot select an arbitrary wake or grant lifecycle authority.
    #[doc(hidden)]
    pub fn owned_async_wake_frontier_for_test(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
    ) -> Result<
        (TemporalWakeSummary, TemporalFrontierSnapshot),
        SignalConditionalServiceExecutionDenial,
    > {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.conditional_owned_async_wake_frontier_for_test(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
            )
        })
    }
}
