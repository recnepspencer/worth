//! A stage its request declared single: that request's own nested stages,
//! which narrow nothing, run on its meter instead of setting up another.
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use super::{KernelContext, KernelState, ACTIVE_METER};
use crate::authority::{
    ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial, StageOrigin,
};

thread_local! {
    static SINGLE_STAGES: RefCell<Vec<(StageOrigin, Rc<RefCell<KernelState>>)>> =
        const { RefCell::new(Vec::new()) };
}

/// Ends the declaration when its stage's kernel returns or unwinds.
pub(crate) struct SingleStageGuard;

impl Drop for SingleStageGuard {
    fn drop(&mut self) {
        SINGLE_STAGES.with(|stages| stages.borrow_mut().pop());
    }
}

impl KernelContext<'_, '_> {
    /// Declare this active stage the single stage of `origin`'s request.
    pub(crate) fn declare_single_stage(&self, origin: StageOrigin) -> SingleStageGuard {
        SINGLE_STAGES.with(|stages| stages.borrow_mut().push((origin, Rc::clone(&self.meter))));
        SingleStageGuard
    }
}

/// The innermost active stage, when `origin` declared it its single stage.
/// Any other scope entered since, even of the same request, hides it.
pub(crate) fn joined_stage<'a, 'lease>(origin: &StageOrigin) -> Option<KernelContext<'a, 'lease>> {
    let active = ACTIVE_METER.with(|active| active.borrow().last().cloned())?;
    let declared = SINGLE_STAGES.with(|stages| {
        stages
            .borrow()
            .last()
            .is_some_and(|(owner, meter)| owner == origin && Rc::ptr_eq(meter, &active))
    });
    declared.then(|| KernelContext {
        meter: active,
        _lease: PhantomData,
    })
}

impl KernelContext<'_, '_> {
    /// Whether a nested `ceiling` would leave this stage's remaining work as it is.
    pub(crate) fn leaves_ceiling(&self, ceiling: Option<u64>) -> bool {
        let meter = self.meter.borrow();
        let remaining = meter
            .limits
            .ceiling
            .map(|own| own.saturating_sub(meter.work()));
        ceiling.is_none_or(|nested| remaining.is_some_and(|remaining| nested >= remaining))
    }
}

impl KernelContext<'_, '_> {
    /// Holds a joined measured result's reservation on this stage's lineage,
    /// as a scope's admission takes it over: a foreign reservation is refused.
    pub(crate) fn hold_joined(
        &self,
        memory: &mut ExecutionMemoryReservation,
        lease: Option<&ExecutionResourceLease<'_>>,
        bytes: u64,
    ) -> Result<(), LeaseDenial> {
        let limits = self.meter.borrow().limits.clone();
        memory.take_over(lease, limits.serial_memory(), bytes)?;
        memory.track_activity(&limits);
        Ok(())
    }
}
