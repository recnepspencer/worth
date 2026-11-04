//! The original admission charge follows every surviving seal verification share.
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};

#[derive(Debug)]
pub(super) struct HomeTransitionAdmissionBacking {
    bytes: u64,
    memory: Option<ExecutionMemoryReservation>,
}

/// Read-only observation of this Home's original seal backing. It holds no
/// admission authority, execution reservation, runtime port or mutation lease.
#[derive(Clone)]
pub struct ApplicationHomeTransitionAdmissionObservation {
    backing: Weak<HomeTransitionAdmissionBacking>,
}
impl ApplicationHomeTransitionAdmissionObservation {
    pub fn held_memory_bytes(&self) -> u64 {
        self.backing.upgrade().map_or(0, |backing| {
            if backing.memory.is_some() {
                backing.bytes
            } else {
                0
            }
        })
    }
}

impl HomeTransitionAdmissionBacking {
    pub(super) fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, WorkCeilingDenial> {
        let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("the fixed admission backing layout fits the target pointer width");
        let bytes = (layout.pad_to_align().size()
            + worth_proof::HomeProgramTransitionOwner::retained_occurrence_bytes())
            as u64;
        let memory = request.reserve_retained_memory(bytes)?;
        Ok(Arc::new(Self { bytes, memory }))
    }

    pub(super) fn observation(
        backing: &Arc<Self>,
    ) -> ApplicationHomeTransitionAdmissionObservation {
        ApplicationHomeTransitionAdmissionObservation {
            backing: Arc::downgrade(backing),
        }
    }
}
