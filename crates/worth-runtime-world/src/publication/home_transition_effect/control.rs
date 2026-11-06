//! One receiver-armed interruption, with no live owner or filesystem custody.
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldHomeTransitionEffectInterruption {
    RefuseHeadMovement,
    UnwindAfterHeadMovement,
}

#[derive(Debug)]
pub(crate) struct HomeTransitionEffectControlState {
    mode: WorldHomeTransitionEffectInterruption,
    armed: AtomicBool,
    fired: AtomicBool,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

// The concrete Arc header/payload layout is also charged to the catalog while
// its one optional slot retains the state. It contains no indirect allocations.
#[repr(C)]
struct ControlAllocation {
    _counts: [AtomicUsize; 2],
    _state: HomeTransitionEffectControlState,
}

/// Dropping the guard disarms its original receiver's one-shot interruption.
/// Retaining it retains only its allocation, never a Home lease or port.
#[derive(Debug)]
pub struct WorldHomeTransitionEffectControl {
    state: Arc<HomeTransitionEffectControlState>,
}
impl WorldHomeTransitionEffectControl {
    pub fn was_triggered(&self) -> bool {
        self.state.fired.load(Ordering::Acquire)
    }
}
impl Drop for WorldHomeTransitionEffectControl {
    fn drop(&mut self) {
        self.state.armed.store(false, Ordering::Release);
    }
}

impl HomeTransitionEffectControlState {
    pub(crate) const fn allocation_bytes() -> usize {
        std::mem::size_of::<ControlAllocation>()
    }

    pub(crate) fn prepare(
        request: &worth_execution::ExecutionRequest<'_, '_>,
        mode: WorldHomeTransitionEffectInterruption,
    ) -> Result<Arc<Self>, super::WorldHomeTransitionEffectDenial> {
        let bytes = u64::try_from(Self::allocation_bytes())
            .map_err(|_| super::WorldHomeTransitionEffectDenial::RetentionCapacity)?;
        let memory = request.reserve_retained_memory(bytes).map_err(|cause| {
            super::WorldHomeTransitionEffectDenial::Medium(
                crate::durability::WorldDurableMediumDenial::Request(cause),
            )
        })?;
        Ok(Arc::new(Self {
            mode,
            armed: AtomicBool::new(true),
            fired: AtomicBool::new(false),
            _memory: memory,
        }))
    }
    pub(crate) fn guard(self: &Arc<Self>) -> WorldHomeTransitionEffectControl {
        WorldHomeTransitionEffectControl {
            state: self.clone(),
        }
    }
    pub(crate) fn is_armed(&self) -> bool {
        self.armed.load(Ordering::Acquire)
    }
    pub(crate) fn fire(&self, mode: WorldHomeTransitionEffectInterruption) -> bool {
        if self.mode == mode && self.armed.swap(false, Ordering::AcqRel) {
            self.fired.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }
}
