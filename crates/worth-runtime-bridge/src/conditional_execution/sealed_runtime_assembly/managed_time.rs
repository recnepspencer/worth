use super::super::*;

impl BridgeSealedRuntimeAssembly {
    pub fn regenerate_managed_interval_wake(
        &self,
        binding: &BridgeManagedClockBinding,
        wake: &BridgeManagedDueWake,
    ) -> Result<worth_signal::facade::IntervalWakeRegeneration, BridgeManagedTemporalDenial> {
        self.runtime
            .execution_view()
            .regenerate_managed_interval_wake(binding, wake)
    }

    pub fn compact_managed_retired_wakes(
        &self,
        binding: &BridgeManagedClockBinding,
        retained_limit: u32,
        max_expired: u32,
    ) -> Result<
        worth_signal::facade::TemporalRetiredWakeCompactionReport,
        BridgeManagedTemporalDenial,
    > {
        self.runtime.execution_view().compact_managed_retired_wakes(
            binding,
            retained_limit,
            max_expired,
        )
    }
}
