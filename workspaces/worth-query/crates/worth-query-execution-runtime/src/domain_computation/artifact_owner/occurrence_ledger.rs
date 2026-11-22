use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub struct WorthQueryArtifactOccurrenceLedger {
    produced_artifact_count: AtomicUsize,
    retained_artifact_count: AtomicUsize,
    disposed_artifact_count: AtomicUsize,
    retained_bytes: AtomicUsize,
}

#[derive(Clone)]
pub struct WorthQueryArtifactOccurrenceScope {
    call: Arc<WorthQueryArtifactOccurrenceLedger>,
    managed_run: Arc<WorthQueryArtifactOccurrenceLedger>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryArtifactOccurrenceSnapshot {
    produced_artifact_count: usize,
    retained_artifact_count: usize,
    disposed_artifact_count: usize,
    retained_bytes: usize,
}

impl WorthQueryArtifactOccurrenceLedger {
    pub fn record_produced(&self, retained_bytes: usize) {
        self.produced_artifact_count.fetch_add(1, Ordering::AcqRel);
        self.retained_artifact_count.fetch_add(1, Ordering::AcqRel);
        self.retained_bytes
            .fetch_add(retained_bytes, Ordering::AcqRel);
    }

    pub fn record_disposed(&self, retained_bytes: usize) {
        decrement(&self.retained_artifact_count, 1);
        decrement(&self.retained_bytes, retained_bytes);
        self.disposed_artifact_count.fetch_add(1, Ordering::AcqRel);
    }

    pub fn snapshot(&self) -> WorthQueryArtifactOccurrenceSnapshot {
        WorthQueryArtifactOccurrenceSnapshot {
            produced_artifact_count: self.produced_artifact_count.load(Ordering::Acquire),
            retained_artifact_count: self.retained_artifact_count.load(Ordering::Acquire),
            disposed_artifact_count: self.disposed_artifact_count.load(Ordering::Acquire),
            retained_bytes: self.retained_bytes.load(Ordering::Acquire),
        }
    }
}

impl WorthQueryArtifactOccurrenceScope {
    pub fn for_managed_run(managed_run: Arc<WorthQueryArtifactOccurrenceLedger>) -> Self {
        Self {
            call: Arc::new(WorthQueryArtifactOccurrenceLedger::default()),
            managed_run,
        }
    }

    pub fn record_produced(&self, retained_bytes: usize) {
        self.call.record_produced(retained_bytes);
        self.managed_run.record_produced(retained_bytes);
    }

    pub fn record_disposed(&self, retained_bytes: usize) {
        self.call.record_disposed(retained_bytes);
        self.managed_run.record_disposed(retained_bytes);
    }

    pub fn call_snapshot(&self) -> WorthQueryArtifactOccurrenceSnapshot {
        self.call.snapshot()
    }
}

impl WorthQueryArtifactOccurrenceSnapshot {
    pub const fn produced_artifact_count(self) -> usize {
        self.produced_artifact_count
    }

    pub const fn retained_artifact_count(self) -> usize {
        self.retained_artifact_count
    }

    pub const fn disposed_artifact_count(self) -> usize {
        self.disposed_artifact_count
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
}

fn decrement(counter: &AtomicUsize, amount: usize) {
    let result = {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let Some(updated_value) = current.checked_sub(amount) else {
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
    };
    debug_assert!(result.is_ok(), "artifact occurrence accounting underflow");
}
