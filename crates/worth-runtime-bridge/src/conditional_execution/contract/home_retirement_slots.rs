//! Original takeable owner slots; ordinary runtime access requires Serving.
use crate::facade::RuntimeBridge;
use std::sync::MutexGuard;
use worth_signal::facade::SignalRuntime;
type Signal = SignalRuntime<(), (), (), (), ()>;

pub(in crate::conditional_execution) struct OrdinaryBridgeCustody {
    runtime: Option<RuntimeBridge>,
}
impl OrdinaryBridgeCustody {
    pub(super) fn new(runtime: RuntimeBridge) -> Self {
        Self {
            runtime: Some(runtime),
        }
    }
    pub(in crate::conditional_execution) fn take(&mut self) -> Option<RuntimeBridge> {
        self.runtime.take()
    }
    pub(in crate::conditional_execution) fn restore(&mut self, runtime: RuntimeBridge) {
        assert!(
            self.runtime.is_none(),
            "only the original refused Bridge is restored"
        );
        self.runtime = Some(runtime);
    }
}
impl std::ops::Deref for OrdinaryBridgeCustody {
    type Target = RuntimeBridge;
    fn deref(&self) -> &Self::Target {
        self.runtime
            .as_ref()
            .expect("retired assembly exposes no Bridge service")
    }
}
impl std::ops::DerefMut for OrdinaryBridgeCustody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.runtime
            .as_mut()
            .expect("retired assembly exposes no Bridge service")
    }
}

pub(in crate::conditional_execution) struct SignalRuntimeGuard<'a> {
    guard: MutexGuard<'a, Option<Signal>>,
}
impl<'a> SignalRuntimeGuard<'a> {
    pub(super) fn new(guard: MutexGuard<'a, Option<Signal>>) -> Self {
        Self { guard }
    }
}
impl std::ops::Deref for SignalRuntimeGuard<'_> {
    type Target = Signal;
    fn deref(&self) -> &Self::Target {
        self.guard
            .as_ref()
            .expect("stopped Signal grants no service")
    }
}
impl std::ops::DerefMut for SignalRuntimeGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard
            .as_mut()
            .expect("stopped Signal grants no service")
    }
}

impl super::BridgeOwnedSignalRuntime {
    pub(in crate::conditional_execution) fn try_prepare_home_signal_stop(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_signal::facade::runtime::PreparedInitialEmptySignalStop<(), (), (), (), ()>,
        crate::facade::BridgeHomeRetirementDenial,
    > {
        use crate::facade::BridgeHomeRetirementDenial as Denial;
        let lowerings = self
            .conditional_lowerings
            .try_read()
            .map_err(|_| Denial::Busy)?;
        let targets = self
            .owned_conditional_targets
            .try_read()
            .map_err(|_| Denial::Busy)?;
        let declarations = self
            .async_declarations
            .try_read()
            .map_err(|_| Denial::Busy)?;
        let clocks = self
            .managed_clock_lanes
            .try_lock()
            .map_err(|_| Denial::Busy)?;
        let _retention = self.retention.try_home_stop_guard()?;
        let _execution = self
            .bridge
            .execution_basis_reservations
            .try_home_stop_guard()?;
        let allocations = self
            .bridge
            .correspondence_allocations
            .try_read()
            .map_err(|_| Denial::Busy)?;
        if !lowerings.is_empty_for_home_stop()
            || !targets.is_empty_for_home_stop()
            || !declarations.is_empty()
            || !clocks.is_empty()
            || !allocations.is_empty()
            || !self
                .bridge
                .semantic_dependency_registry
                .is_empty_for_home_stop()
            || self.baseline_semantic_dependency_count != 0
        {
            return Err(Denial::Unsupported);
        }
        let slot = self.signal_runtime.get_mut().map_err(|_| {
            Denial::Signal(worth_signal::facade::runtime::SignalInitialEmptyStopDenial::Poisoned)
        })?;
        let runtime = slot.take().ok_or(Denial::Unsupported)?;
        match runtime.try_prepare_initial_empty_stop(request) {
            Ok(prepared) => Ok(prepared),
            Err(refused) => {
                let reason = refused.reason();
                *slot = Some(refused.into_runtime());
                Err(Denial::Signal(reason))
            }
        }
    }
}
#[cfg(test)]
#[path = "home_retirement_slots_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
fn collect_owned_inline_type_sizes(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<OrdinaryBridgeCustody>("crates/worth-runtime-bridge/src/conditional_execution/contract/home_retirement_slots.rs", "OrdinaryBridgeCustody", 128 * 1024);
        owner.field::<Option<RuntimeBridge>>("runtime");
    }
    {
        let mut owner = census.owner::<SignalRuntimeGuard<'static>>("crates/worth-runtime-bridge/src/conditional_execution/contract/home_retirement_slots.rs", "SignalRuntimeGuard", 128 * 1024);
        owner.field::<MutexGuard<'static, Option<Signal>>>("guard");
    }
}
