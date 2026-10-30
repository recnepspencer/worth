use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use super::{LifecycleGeneration, RuntimeIdentity};

static PROCESS_COUNTERS: ProcessCounterCells = ProcessCounterCells::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCounterSnapshot {
    runtime_identity: RuntimeIdentity,
    lifecycle_generation: LifecycleGeneration,
    admission_attempts: u64,
    admitted_incarnations: u64,
    observation_acquisitions: u64,
    active_observations: u64,
    lifecycle_observations: u64,
    capability_observations: u64,
    explicit_closes: u64,
    explicit_aborts: u64,
    panic_terminations: u64,
    unexpected_drops: u64,
}

impl RuntimeCounterSnapshot {
    pub const fn runtime_identity(self) -> RuntimeIdentity {
        self.runtime_identity
    }

    pub const fn lifecycle_generation(self) -> LifecycleGeneration {
        self.lifecycle_generation
    }

    pub const fn admission_attempts(self) -> u64 {
        self.admission_attempts
    }

    pub const fn admitted_incarnations(self) -> u64 {
        self.admitted_incarnations
    }

    pub const fn observation_acquisitions(self) -> u64 {
        self.observation_acquisitions
    }

    pub const fn active_observations(self) -> u64 {
        self.active_observations
    }

    pub const fn lifecycle_observations(self) -> u64 {
        self.lifecycle_observations
    }

    pub const fn capability_observations(self) -> u64 {
        self.capability_observations
    }

    pub const fn explicit_closes(self) -> u64 {
        self.explicit_closes
    }

    pub const fn explicit_aborts(self) -> u64 {
        self.explicit_aborts
    }

    pub const fn panic_terminations(self) -> u64 {
        self.panic_terminations
    }

    pub const fn unexpected_drops(self) -> u64 {
        self.unexpected_drops
    }

    pub const fn physical_owner_count(self) -> u64 {
        0
    }

    pub const fn physical_operation_attempts(self) -> u64 {
        0
    }

    pub const fn publication_attempts(self) -> u64 {
        0
    }

    pub const fn media_operations(self) -> u64 {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessRuntimeCounterSnapshot {
    admission_attempts: u64,
    admitted_incarnations: u64,
    admission_denials: u64,
    admission_cancellations: u64,
    admission_panics_before_return: u64,
    observation_acquisitions: u64,
    active_observations: u64,
    lifecycle_observations: u64,
    capability_observations: u64,
    explicit_closes: u64,
    explicit_aborts: u64,
    panic_terminations: u64,
    unexpected_drops: u64,
}

macro_rules! process_counter_accessors {
    ($($name:ident),+ $(,)?) => {
        $(pub const fn $name(self) -> u64 { self.$name })+
    };
}

impl ProcessRuntimeCounterSnapshot {
    process_counter_accessors!(
        admission_attempts,
        admitted_incarnations,
        admission_denials,
        admission_cancellations,
        admission_panics_before_return,
        observation_acquisitions,
        active_observations,
        lifecycle_observations,
        capability_observations,
        explicit_closes,
        explicit_aborts,
        panic_terminations,
        unexpected_drops,
    );

    pub const fn physical_owner_count(self) -> u64 {
        0
    }

    pub const fn physical_operation_attempts(self) -> u64 {
        0
    }

    pub const fn publication_attempts(self) -> u64 {
        0
    }

    pub const fn media_operations(self) -> u64 {
        0
    }
}

pub(crate) struct RuntimeDiagnostics {
    cells: Arc<RuntimeCounterCells>,
}

impl RuntimeDiagnostics {
    pub(crate) fn admitted(runtime_identity: RuntimeIdentity) -> Self {
        Self {
            cells: Arc::new(RuntimeCounterCells::new(runtime_identity)),
        }
    }

    pub(crate) fn counter_cells(&self) -> Arc<RuntimeCounterCells> {
        Arc::clone(&self.cells)
    }

    pub(crate) fn snapshot(
        &self,
        lifecycle_generation: LifecycleGeneration,
    ) -> RuntimeCounterSnapshot {
        self.cells.snapshot(lifecycle_generation)
    }

    pub(crate) fn record_capability_observations(&self, family_count: u64) {
        self.cells.record_capability_observations(family_count);
    }
}

pub(crate) struct RuntimeCounterCells {
    runtime_identity: RuntimeIdentity,
    observation_acquisitions: AtomicU64,
    active_observations: AtomicU64,
    lifecycle_observations: AtomicU64,
    capability_observations: AtomicU64,
    explicit_closes: AtomicU64,
    explicit_aborts: AtomicU64,
    panic_terminations: AtomicU64,
    unexpected_drops: AtomicU64,
}

impl RuntimeCounterCells {
    fn new(runtime_identity: RuntimeIdentity) -> Self {
        Self {
            runtime_identity,
            observation_acquisitions: AtomicU64::new(0),
            active_observations: AtomicU64::new(0),
            lifecycle_observations: AtomicU64::new(0),
            capability_observations: AtomicU64::new(0),
            explicit_closes: AtomicU64::new(0),
            explicit_aborts: AtomicU64::new(0),
            panic_terminations: AtomicU64::new(0),
            unexpected_drops: AtomicU64::new(0),
        }
    }

    pub(crate) fn acquire_observation(&self) {
        increment(&self.observation_acquisitions, 1);
        increment(&self.active_observations, 1);
        PROCESS_COUNTERS.acquire_observation();
    }

    pub(crate) fn release_observation(&self) {
        decrement(&self.active_observations);
        PROCESS_COUNTERS.release_observation();
    }

    pub(crate) fn record_lifecycle_observation(&self) {
        increment(&self.lifecycle_observations, 1);
        increment(&PROCESS_COUNTERS.lifecycle_observations, 1);
    }

    pub(crate) fn record_capability_observations(&self, family_count: u64) {
        increment(&self.capability_observations, family_count);
        increment(&PROCESS_COUNTERS.capability_observations, family_count);
    }

    pub(crate) fn record_explicit_close(&self) {
        increment(&self.explicit_closes, 1);
        increment(&PROCESS_COUNTERS.explicit_closes, 1);
    }

    pub(crate) fn record_explicit_abort(&self) {
        increment(&self.explicit_aborts, 1);
        increment(&PROCESS_COUNTERS.explicit_aborts, 1);
    }

    pub(crate) fn record_panic_termination(&self) {
        increment(&self.panic_terminations, 1);
        increment(&PROCESS_COUNTERS.panic_terminations, 1);
    }

    pub(crate) fn record_unexpected_drop(&self) {
        increment(&self.unexpected_drops, 1);
        increment(&PROCESS_COUNTERS.unexpected_drops, 1);
    }

    pub(crate) fn snapshot(
        &self,
        lifecycle_generation: LifecycleGeneration,
    ) -> RuntimeCounterSnapshot {
        RuntimeCounterSnapshot {
            runtime_identity: self.runtime_identity,
            lifecycle_generation,
            admission_attempts: 1,
            admitted_incarnations: 1,
            observation_acquisitions: load(&self.observation_acquisitions),
            active_observations: load(&self.active_observations),
            lifecycle_observations: load(&self.lifecycle_observations),
            capability_observations: load(&self.capability_observations),
            explicit_closes: load(&self.explicit_closes),
            explicit_aborts: load(&self.explicit_aborts),
            panic_terminations: load(&self.panic_terminations),
            unexpected_drops: load(&self.unexpected_drops),
        }
    }
}

struct ProcessCounterCells {
    admission_attempts: AtomicU64,
    admitted_incarnations: AtomicU64,
    admission_denials: AtomicU64,
    admission_cancellations: AtomicU64,
    admission_panics_before_return: AtomicU64,
    observation_acquisitions: AtomicU64,
    active_observations: AtomicU64,
    lifecycle_observations: AtomicU64,
    capability_observations: AtomicU64,
    explicit_closes: AtomicU64,
    explicit_aborts: AtomicU64,
    panic_terminations: AtomicU64,
    unexpected_drops: AtomicU64,
}

impl ProcessCounterCells {
    const fn new() -> Self {
        Self {
            admission_attempts: AtomicU64::new(0),
            admitted_incarnations: AtomicU64::new(0),
            admission_denials: AtomicU64::new(0),
            admission_cancellations: AtomicU64::new(0),
            admission_panics_before_return: AtomicU64::new(0),
            observation_acquisitions: AtomicU64::new(0),
            active_observations: AtomicU64::new(0),
            lifecycle_observations: AtomicU64::new(0),
            capability_observations: AtomicU64::new(0),
            explicit_closes: AtomicU64::new(0),
            explicit_aborts: AtomicU64::new(0),
            panic_terminations: AtomicU64::new(0),
            unexpected_drops: AtomicU64::new(0),
        }
    }

    fn acquire_observation(&self) {
        increment(&self.observation_acquisitions, 1);
        increment(&self.active_observations, 1);
    }

    fn release_observation(&self) {
        decrement(&self.active_observations);
    }

    fn snapshot(&self) -> ProcessRuntimeCounterSnapshot {
        ProcessRuntimeCounterSnapshot {
            admission_attempts: load(&self.admission_attempts),
            admitted_incarnations: load(&self.admitted_incarnations),
            admission_denials: load(&self.admission_denials),
            admission_cancellations: load(&self.admission_cancellations),
            admission_panics_before_return: load(&self.admission_panics_before_return),
            observation_acquisitions: load(&self.observation_acquisitions),
            active_observations: load(&self.active_observations),
            lifecycle_observations: load(&self.lifecycle_observations),
            capability_observations: load(&self.capability_observations),
            explicit_closes: load(&self.explicit_closes),
            explicit_aborts: load(&self.explicit_aborts),
            panic_terminations: load(&self.panic_terminations),
            unexpected_drops: load(&self.unexpected_drops),
        }
    }
}

pub(crate) fn record_admission_attempt() {
    increment(&PROCESS_COUNTERS.admission_attempts, 1);
}

pub(crate) fn record_admitted_incarnation() {
    increment(&PROCESS_COUNTERS.admitted_incarnations, 1);
}

pub(crate) fn record_admission_denial() {
    increment(&PROCESS_COUNTERS.admission_denials, 1);
}

pub(crate) fn record_admission_cancellation() {
    increment(&PROCESS_COUNTERS.admission_cancellations, 1);
}

pub(crate) fn record_admission_panic_before_return() {
    increment(&PROCESS_COUNTERS.admission_panics_before_return, 1);
}

pub(crate) fn process_counter_snapshot() -> ProcessRuntimeCounterSnapshot {
    PROCESS_COUNTERS.snapshot()
}

fn increment(counter: &AtomicU64, amount: u64) {
    {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let next_value = Some(current.saturating_add(amount));
            let Some(updated_value) = next_value else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .expect("the counter update closure always returns a value");
}

fn decrement(counter: &AtomicU64) {
    {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let next_value = current.checked_sub(1);
            let Some(updated_value) = next_value else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .expect("resource release must correspond to an active acquisition");
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Acquire)
}
