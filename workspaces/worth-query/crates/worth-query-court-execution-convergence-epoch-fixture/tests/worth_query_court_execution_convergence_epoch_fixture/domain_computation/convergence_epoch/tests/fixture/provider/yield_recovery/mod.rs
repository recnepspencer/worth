use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub enum FixtureYieldRecoveryArtifact {
    Cooperative,
    DoublePanicking,
}

#[derive(Clone, Default)]
pub struct FixtureYieldRecoveryProbe {
    pub suspension_attempts: Arc<AtomicUsize>,
    pub disposal_attempts: Arc<AtomicUsize>,
    pub destructor_attempts: Arc<AtomicUsize>,
}

impl FixtureYieldRecoveryProbe {
    pub fn suspension_attempt_count(&self) -> usize {
        self.suspension_attempts.load(Ordering::Acquire)
    }

    pub fn disposal_attempt_count(&self) -> usize {
        self.disposal_attempts.load(Ordering::Acquire)
    }

    pub fn destructor_attempt_count(&self) -> usize {
        self.destructor_attempts.load(Ordering::Acquire)
    }

    pub fn attempted_suspension(&self) {
        self.suspension_attempts.fetch_add(1, Ordering::AcqRel);
    }

    pub fn attempted_disposal(&self) {
        self.disposal_attempts.fetch_add(1, Ordering::AcqRel);
    }

    pub fn attempted_destructor(&self) {
        self.destructor_attempts.fetch_add(1, Ordering::AcqRel);
    }
}
