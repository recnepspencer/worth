use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct PhysicalMutationPauseGate {
    checkpoint: PhysicalMutationCheckpoint,
    shared: Arc<PhysicalMutationPauseState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalMutationCheckpoint {
    BeforeEffectCutover,
    AfterGroupSeal,
    AfterWalDurability,
    /// A writeback claim and scheduler admission are live, immediately before
    /// the physical writeback effect begins.
    AfterWritebackAdmissionBeforeEffect,
    DuringDataSettlement,
    AfterDataSettlement,
    DuringRootPublication,
    BeforeTerminalFinalization,
    RuntimeClosingMarked,
}

struct PhysicalMutationPauseState {
    progress: Mutex<PhysicalMutationPauseProgress>,
    changed: Condvar,
}

struct PhysicalMutationPauseProgress {
    arrived: bool,
    released: bool,
}

pub(super) struct PhysicalMutationYieldpointOwner {
    gates: Mutex<HashMap<PhysicalMutationCheckpoint, PhysicalMutationPauseGate>>,
}

impl PhysicalMutationYieldpointOwner {
    pub(super) fn new() -> Self {
        Self {
            gates: Mutex::new(HashMap::new()),
        }
    }

    pub(super) fn install(
        &self,
        checkpoint: PhysicalMutationCheckpoint,
    ) -> PhysicalMutationPauseGate {
        let gate = PhysicalMutationPauseGate::new(checkpoint);
        self.gates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(checkpoint, gate.clone());
        gate
    }

    pub(super) fn pause(&self, checkpoint: PhysicalMutationCheckpoint) {
        let gate = self
            .gates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&checkpoint)
            .cloned();
        if let Some(gate) = gate {
            gate.arrive_and_wait();
        }
    }
}

impl PhysicalMutationPauseGate {
    fn new(checkpoint: PhysicalMutationCheckpoint) -> Self {
        Self {
            checkpoint,
            shared: Arc::new(PhysicalMutationPauseState {
                progress: Mutex::new(PhysicalMutationPauseProgress {
                    arrived: false,
                    released: false,
                }),
                changed: Condvar::new(),
            }),
        }
    }

    pub const fn checkpoint(&self) -> PhysicalMutationCheckpoint {
        self.checkpoint
    }

    pub fn await_arrival(&self) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut progress = self
            .shared
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !progress.arrived {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            let (next, wait) = self
                .shared
                .changed
                .wait_timeout(progress, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            progress = next;
            if wait.timed_out() && !progress.arrived {
                return false;
            }
        }
        true
    }

    pub fn release(&self) {
        let mut progress = self
            .shared
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        progress.released = true;
        self.shared.changed.notify_all();
    }

    fn arrive_and_wait(&self) {
        let mut progress = self
            .shared
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        progress.arrived = true;
        self.shared.changed.notify_all();
        while !progress.released {
            progress = self
                .shared
                .changed
                .wait(progress)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }
}
