use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::{Duration, Instant};

use crate::physical_runtime::work::PhysicalCheckpointWorkAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCheckpointStep {
    CandidateCreation,
    CandidateAppend,
    CandidateBindingCompactionHeader,
    CandidateBindingRecord,
    CandidateFooter,
    CandidateSynchronization,
    CandidatePublication,
    NamespaceSynchronization,
}

impl PhysicalCheckpointStep {
    const fn index(self) -> usize {
        match self {
            Self::CandidateCreation => 0,
            Self::CandidateAppend => 1,
            Self::CandidateBindingCompactionHeader => 2,
            Self::CandidateBindingRecord => 3,
            Self::CandidateFooter => 4,
            Self::CandidateSynchronization => 5,
            Self::CandidatePublication => 6,
            Self::NamespaceSynchronization => 7,
        }
    }
}

pub struct PhysicalCheckpointPauseGate {
    step: PhysicalCheckpointStep,
    shared: Arc<PauseState>,
}

struct PauseState {
    progress: Mutex<PauseProgress>,
    changed: Condvar,
}

struct PauseProgress {
    arrived: bool,
    released: bool,
}

pub(crate) struct PhysicalCheckpointYieldpointOwner {
    gates: [Mutex<Option<Weak<PauseState>>>; 8],
}

impl PhysicalCheckpointYieldpointOwner {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            gates: [
                Mutex::new(None),
                Mutex::new(None),
                Mutex::new(None),
                Mutex::new(None),
                Mutex::new(None),
                Mutex::new(None),
                Mutex::new(None),
                Mutex::new(None),
            ],
        })
    }

    pub(super) fn install(&self, step: PhysicalCheckpointStep) -> PhysicalCheckpointPauseGate {
        let gate = PhysicalCheckpointPauseGate {
            step,
            shared: Arc::new(PauseState {
                progress: Mutex::new(PauseProgress {
                    arrived: false,
                    released: false,
                }),
                changed: Condvar::new(),
            }),
        };
        *self.gates[step.index()]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::downgrade(&gate.shared));
        gate
    }

    pub(super) fn pause_after(&self, action: PhysicalCheckpointWorkAction) {
        let Some(step) = step_for_action(action) else {
            return;
        };
        self.pause_after_step(step);
    }

    pub(super) fn pause_after_step(&self, step: PhysicalCheckpointStep) {
        let shared = self.gates[step.index()]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .and_then(Weak::upgrade);
        if let Some(shared) = shared {
            let mut progress = shared
                .progress
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            progress.arrived = true;
            shared.changed.notify_all();
            while !progress.released {
                progress = shared
                    .changed
                    .wait(progress)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        }
    }
}

impl PhysicalCheckpointPauseGate {
    pub const fn step(&self) -> PhysicalCheckpointStep {
        self.step
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
}

impl Drop for PhysicalCheckpointPauseGate {
    fn drop(&mut self) {
        self.release();
    }
}

const fn step_for_action(action: PhysicalCheckpointWorkAction) -> Option<PhysicalCheckpointStep> {
    match action {
        PhysicalCheckpointWorkAction::CreateCandidate { .. } => {
            Some(PhysicalCheckpointStep::CandidateCreation)
        }
        PhysicalCheckpointWorkAction::AppendCandidate { .. } => None,
        PhysicalCheckpointWorkAction::SynchronizeCandidate => {
            Some(PhysicalCheckpointStep::CandidateSynchronization)
        }
        PhysicalCheckpointWorkAction::PublishCandidate => {
            Some(PhysicalCheckpointStep::CandidatePublication)
        }
        PhysicalCheckpointWorkAction::SynchronizeNamespace => {
            Some(PhysicalCheckpointStep::NamespaceSynchronization)
        }
        PhysicalCheckpointWorkAction::RemoveCandidate => None,
    }
}
