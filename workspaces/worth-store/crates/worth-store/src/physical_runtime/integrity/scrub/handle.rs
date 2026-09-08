use super::{cancellation::ScrubRegistration, *};
use crate::physical_runtime::record_serving::{
    residency::RecordFramePorts, CanonicalRecordReadPort,
};
use crate::physical_runtime::{
    lifecycle::{LifecycleState, ObservedLifecyclePhase},
    LifecycleGeneration, RuntimeIdentity,
};
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::time::Instant;

/// Opaque continuation bound to the same handle, runtime incarnation, lifecycle
/// generation and next target. It cannot retarget a request or replay a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityScrubResume {
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
    handle: u64,
    next_target: usize,
}

pub struct ManagedPhysicalIntegrityScrubHandle {
    request: ManagedPhysicalIntegrityScrubRequest,
    read: CanonicalRecordReadPort,
    frames: RecordFramePorts,
    lifecycle: Arc<LifecycleState>,
    registration: Arc<ScrubRegistration>,
    window: Arc<Mutex<()>>,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
    identity: u64,
    next_target: usize,
    paused: bool,
    started: Instant,
    counters: PhysicalIntegrityScrubCounters,
    terminal: Option<ManagedPhysicalIntegrityScrubProgress>,
    validator: worth_store_physical_integrity::PhysicalIntegrityScrubValidator,
    checkpoint_source: Option<worth_store_physical_backend::InspectionSourceVersion>,
}

impl ManagedPhysicalIntegrityScrubHandle {
    pub(in crate::physical_runtime) fn start(
        request: ManagedPhysicalIntegrityScrubRequest,
        owner: &PhysicalIntegrityScrubOwner,
        read: CanonicalRecordReadPort,
        frames: RecordFramePorts,
        lifecycle: Arc<LifecycleState>,
        runtime: RuntimeIdentity,
    ) -> Result<Self, PhysicalIntegrityScrubRequestDenial> {
        let snapshot = lifecycle.snapshot();
        if snapshot.phase != ObservedLifecyclePhase::RecordServing {
            return Err(PhysicalIntegrityScrubRequestDenial::RuntimeClosed);
        }
        let (identity, registration) = owner.register()?;
        let counters = PhysicalIntegrityScrubCounters {
            declared_targets: request.targets.len() as u64,
            ..Default::default()
        };
        Ok(Self {
            request,
            read,
            frames,
            lifecycle,
            registration,
            window: owner.window.clone(),
            runtime,
            generation: snapshot.generation,
            identity,
            next_target: 0,
            paused: false,
            started: Instant::now(),
            counters,
            terminal: None,
            validator: worth_store_physical_integrity::PhysicalIntegrityScrubValidator::new(),
            checkpoint_source: None,
        })
    }

    pub fn cancellation(&self) -> PhysicalIntegrityScrubCancellation {
        PhysicalIntegrityScrubCancellation {
            registration: self.registration.clone(),
        }
    }
    pub const fn counters(&self) -> PhysicalIntegrityScrubCounters {
        self.counters
    }
    pub fn pause(&mut self) -> PhysicalIntegrityScrubResume {
        self.paused = true;
        PhysicalIntegrityScrubResume {
            runtime: self.runtime,
            generation: self.generation,
            handle: self.identity,
            next_target: self.next_target,
        }
    }
    pub fn resume(
        &mut self,
        token: PhysicalIntegrityScrubResume,
    ) -> Result<(), PhysicalIntegrityScrubRequestDenial> {
        if self.terminal.is_some()
            || !self.paused
            || token.runtime != self.runtime
            || token.generation != self.generation
            || token.handle != self.identity
            || token.next_target != self.next_target
            || self.gate().is_some()
        {
            return Err(PhysicalIntegrityScrubRequestDenial::RuntimeScopeMismatch);
        }
        self.paused = false;
        Ok(())
    }

    /// Pulls at most one bounded observation. There is no hidden result queue,
    /// callback validator, preloaded snapshot, or detached background worker.
    pub fn next_window(&mut self) -> ManagedPhysicalIntegrityScrubProgress {
        if let Some(terminal) = self.terminal {
            return terminal;
        }
        if let Some(terminal) = self.gate() {
            return self.finish(terminal);
        }
        if self.paused {
            return ManagedPhysicalIntegrityScrubProgress::Paused;
        }
        if self.next_target == self.request.targets.len() {
            let outcome = if self.counters.indeterminate_windows
                + self.counters.unknown_windows
                + self.counters.unsupported_windows
                == 0
            {
                ManagedPhysicalIntegrityScrubProgress::Completed(self.counters)
            } else {
                ManagedPhysicalIntegrityScrubProgress::Indeterminate(self.counters)
            };
            return self.finish(outcome);
        }
        let window = self.window.clone();
        let _permit = match window.try_lock() {
            Ok(permit) => permit,
            Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                return self.defer(PhysicalIntegrityScrubDeferral::AnotherWindowActive)
            }
        };
        if let Some(terminal) = self.gate() {
            return self.finish(terminal);
        }
        let target = self.request.targets[self.next_target];
        let allocation = match self.frames.begin_operation(
            worth_store_buffer_pool::PhysicalOperationAllocationScope::Scrub,
            std::num::NonZeroU64::new(target.range().length() as u64)
                .expect("bounded nonempty target"),
        ) {
            Ok(allocation) => allocation,
            Err(_) => return self.defer(PhysicalIntegrityScrubDeferral::Allocation),
        };
        self.counters.peak_allocation_bytes =
            self.counters.peak_allocation_bytes.max(allocation.bytes());
        let destination = vec![0; target.range().length() as usize].into_boxed_slice();
        let evidence = match self.read.inspect(target.range(), destination) {
            Ok(evidence) => evidence,
            Err(cause) => {
                drop(allocation);
                return self.defer(PhysicalIntegrityScrubDeferral::SchedulerOrDependency(cause));
            }
        };
        let observation = super::window_inspection::inspect(
            self.next_target as u64,
            target.scope(),
            evidence,
            self.started.elapsed() >= self.request.deadline,
            &mut self.counters,
            &mut self.validator,
            &mut self.checkpoint_source,
        );
        drop(allocation);
        self.next_target += 1;
        // A cancellation arriving after I/O preserves this real completed window.
        // The next pull emits the terminal state with the same counters.
        ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation)
    }

    fn gate(&self) -> Option<ManagedPhysicalIntegrityScrubProgress> {
        use ManagedPhysicalIntegrityScrubProgress as Progress;
        let snapshot = self.lifecycle.snapshot();
        if self.registration.closed.load(Ordering::Acquire)
            || snapshot.phase != ObservedLifecyclePhase::RecordServing
        {
            return Some(Progress::Closed(self.counters));
        }
        if snapshot.generation != self.generation {
            return Some(Progress::StaleRuntimeGeneration(self.counters));
        }
        if self.registration.cancelled.load(Ordering::Acquire) {
            return Some(Progress::Cancelled(self.counters));
        }
        if self.started.elapsed() >= self.request.deadline {
            return Some(Progress::DeadlineExceeded(self.counters));
        }
        None
    }
    fn defer(
        &mut self,
        cause: PhysicalIntegrityScrubDeferral,
    ) -> ManagedPhysicalIntegrityScrubProgress {
        self.counters.deferred_windows = self.counters.deferred_windows.saturating_add(1);
        ManagedPhysicalIntegrityScrubProgress::Deferred(cause)
    }
    fn finish(
        &mut self,
        terminal: ManagedPhysicalIntegrityScrubProgress,
    ) -> ManagedPhysicalIntegrityScrubProgress {
        self.request.targets = Box::new([]);
        self.checkpoint_source = None;
        self.registration.finished.store(true, Ordering::Release);
        self.terminal = Some(terminal);
        terminal
    }
}

impl Drop for ManagedPhysicalIntegrityScrubHandle {
    fn drop(&mut self) {
        self.registration.finished.store(true, Ordering::Release);
    }
}
