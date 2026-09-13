//! A bounded, owner-keyed, one-shot rehearsal of the two execution boundaries a
//! concurrency proof must stand on: after the Relational leg and before the
//! pre-advance gate, and after that gate and immediately before the Signal
//! advance. Execution consults it only for an owner a test has armed, and a
//! rehearsal never becomes an open-ended wait.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use worth_signal::facade::branch::SignalOwnerCancellationToken;

use crate::identity::RuntimeWorldOwnerIdentity;

/// An unreleased pause fails the arming test by name inside this budget instead
/// of hanging the suite.
const REHEARSAL_RELEASE_BUDGET: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExecutionRehearsalBoundary {
    /// The Relational leg has settled and the pre-advance gate has not run.
    BetweenOwnerEffects,
    /// The pre-advance gate has passed and the Signal owner has not been
    /// contacted.
    SignalAdvance,
}

/// What the parked execution reports to the arming test.
pub(super) enum ReachedExecutionBoundary {
    BetweenOwnerEffects,
    SignalAdvance {
        signal_token: SignalOwnerCancellationToken,
    },
}

struct ArmedRehearsal {
    boundary: ExecutionRehearsalBoundary,
    reached: SyncSender<ReachedExecutionBoundary>,
    signal_advance_entries: Arc<AtomicUsize>,
    fired: bool,
    released: bool,
}

type RehearsalEntries = HashMap<RuntimeWorldOwnerIdentity, ArmedRehearsal>;

fn registry() -> &'static (Mutex<RehearsalEntries>, Condvar) {
    static REGISTRY: OnceLock<(Mutex<RehearsalEntries>, Condvar)> = OnceLock::new();
    REGISTRY.get_or_init(|| (Mutex::new(HashMap::new()), Condvar::new()))
}

fn entries() -> MutexGuard<'static, RehearsalEntries> {
    registry()
        .0
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// The arming handle. Dropping it releases any parked execution and disarms the
/// owner, so a failing assertion can never leave a thread parked.
pub(super) struct ExecutionRehearsal {
    owner: RuntimeWorldOwnerIdentity,
    signal_advance_entries: Arc<AtomicUsize>,
}

impl ExecutionRehearsal {
    pub(super) fn arm(
        owner: RuntimeWorldOwnerIdentity,
        boundary: ExecutionRehearsalBoundary,
        reached: SyncSender<ReachedExecutionBoundary>,
    ) -> Self {
        let signal_advance_entries = Arc::new(AtomicUsize::new(0));
        entries().insert(
            owner,
            ArmedRehearsal {
                boundary,
                reached,
                signal_advance_entries: Arc::clone(&signal_advance_entries),
                fired: false,
                released: false,
            },
        );
        Self {
            owner,
            signal_advance_entries,
        }
    }

    /// Lets the parked execution continue past the armed boundary.
    pub(super) fn release(&self) {
        if let Some(entry) = entries().get_mut(&self.owner) {
            entry.released = true;
        }
        registry().1.notify_all();
    }

    /// How many times this owner's execution reached the Signal advance seam.
    /// Zero is the exact evidence that the Signal owner was never contacted.
    pub(super) fn signal_advance_entries(&self) -> usize {
        self.signal_advance_entries.load(Ordering::Acquire)
    }
}

impl Drop for ExecutionRehearsal {
    fn drop(&mut self) {
        entries().remove(&self.owner);
        registry().1.notify_all();
    }
}

pub(super) fn reach_between_owner_effects(owner: RuntimeWorldOwnerIdentity) {
    reach(
        owner,
        ExecutionRehearsalBoundary::BetweenOwnerEffects,
        || ReachedExecutionBoundary::BetweenOwnerEffects,
    );
}

pub(super) fn reach_signal_advance(
    owner: RuntimeWorldOwnerIdentity,
    signal_token: &SignalOwnerCancellationToken,
) {
    reach(owner, ExecutionRehearsalBoundary::SignalAdvance, || {
        ReachedExecutionBoundary::SignalAdvance {
            signal_token: signal_token.clone(),
        }
    });
}

fn reach(
    owner: RuntimeWorldOwnerIdentity,
    boundary: ExecutionRehearsalBoundary,
    report: impl FnOnce() -> ReachedExecutionBoundary,
) {
    let mut armed = entries();
    let Some(entry) = armed.get_mut(&owner) else {
        return;
    };
    if boundary == ExecutionRehearsalBoundary::SignalAdvance {
        entry.signal_advance_entries.fetch_add(1, Ordering::Release);
    }
    if entry.boundary != boundary || entry.fired {
        return;
    }
    entry.fired = true;
    entry
        .reached
        .send(report())
        .expect("the arming test observes its execution rehearsal boundary");
    park_until_released(armed, owner);
}

fn park_until_released(
    mut armed: MutexGuard<'static, RehearsalEntries>,
    owner: RuntimeWorldOwnerIdentity,
) {
    let deadline = Instant::now() + REHEARSAL_RELEASE_BUDGET;
    loop {
        match armed.get(&owner) {
            None => return,
            Some(entry) if entry.released => return,
            Some(_) => {}
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(
            !remaining.is_zero(),
            "an armed execution rehearsal was never released within its budget"
        );
        let (next, _) = registry()
            .1
            .wait_timeout(armed, remaining)
            .unwrap_or_else(|error| error.into_inner());
        armed = next;
    }
}
