//! Push commit notification and the cheap committed-basis version of a Home.
//!
//! The version is one atomic load of the Home's commit pulse. Watches share one
//! notifier thread per Home: it waits on the pulse and, after each advance,
//! calls every registered callback with no Host or Query lock held. The thread
//! starts with the first watch and stops when the last guard drops or the Home
//! closes; it holds only the pulse, never the Home's runtime.
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::product::WorthQueryCommitPulse;

use super::thread_site::spawn_named;
use crate::application_home::owned_holds::{OwnedRuntimeHold, OwnedRuntimeHolds};
use crate::application_home::WorthQueryApplicationHome;

/// A Home's committed-basis version, from `committed_basis_version()`.
///
/// It changes after every commit that publishes to any branch of the Home
/// (sessions, workflows, previews and raw requests alike) and never
/// otherwise. Compare two values of the same open Home: a reopened Home starts
/// a new sequence, which compares unequal to every value of the old one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct WorthQueryCommittedBasisVersion {
    incarnation: u64,
    published: u64,
}

impl WorthQueryCommittedBasisVersion {
    pub(crate) fn of(pulse: &WorthQueryCommitPulse) -> Self {
        Self {
            incarnation: pulse.incarnation(),
            published: pulse.version(),
        }
    }

    /// Whether anything committed between `earlier` and this version.
    pub fn changed_since(self, earlier: Self) -> bool {
        self != earlier
    }
}

/// Why a commit watch could not start.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryCommitWatchDenial {
    /// The Home closed; nothing more can commit to it.
    HomeClosed,
    /// The operating system refused the notifier thread; nothing registered.
    NotifierUnavailable,
}

/// Keeps one commit callback registered; dropping it unregisters the callback.
///
/// Dropping never blocks: a callback already running finishes, and one the
/// notifier had already picked up for the current commit may still start once
/// as the guard drops; none starts for a later commit. Dropping it from inside
/// its own callback is allowed.
#[must_use = "the callback is unregistered as soon as the guard drops"]
pub struct WorthQueryCommitWatchGuard {
    registry: Arc<CommitWatchRegistry>,
    key: u64,
}

impl std::fmt::Debug for WorthQueryCommitWatchGuard {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WorthQueryCommitWatchGuard(..)")
    }
}

impl Drop for WorthQueryCommitWatchGuard {
    fn drop(&mut self) {
        self.registry.unregister(self.key);
    }
}

type CommitCallback = Arc<dyn Fn() + Send + Sync>;

/// Every commit callback of one Home and the state of its notifier thread.
#[derive(Default)]
pub(crate) struct CommitWatchRegistry {
    state: Mutex<RegistryState>,
}

#[derive(Default)]
struct RegistryState {
    next_key: u64,
    callbacks: BTreeMap<u64, CommitCallback>,
    notifier: Option<WorthQueryCommitPulse>,
    enrolled: bool,
    closed: bool,
}

/// How long the notifier sleeps between checks when nothing wakes it. Guard
/// drops and Home close wake it at once; this bounds a missed wake only.
const NOTIFIER_RECHECK: Duration = Duration::from_secs(1);

impl CommitWatchRegistry {
    /// Registers `callback` to run after each commit that `pulse` reports.
    /// The first watch also enrols the registry in `holds`, so closing the
    /// Home stops the notifier.
    pub(crate) fn watch(
        self: &Arc<Self>,
        holds: &OwnedRuntimeHolds,
        pulse: WorthQueryCommitPulse,
        callback: CommitCallback,
    ) -> Result<WorthQueryCommitWatchGuard, WorthQueryCommitWatchDenial> {
        let mut state = self.lock();
        if state.closed {
            return Err(WorthQueryCommitWatchDenial::HomeClosed);
        }
        if !state.enrolled {
            let hold: Arc<dyn OwnedRuntimeHold + Send + Sync> = self.clone();
            holds.register_shared(Arc::downgrade(&hold));
            state.enrolled = true;
        }
        let key = state.next_key;
        state.next_key = state.next_key.saturating_add(1);
        state.callbacks.insert(key, callback);
        if state.notifier.is_none() {
            // The notifier counts from the version current at registration,
            // so no commit after this call can be missed.
            let seen = pulse.version();
            let registry = Arc::clone(self);
            let notified = pulse.clone();
            if spawn_named("worth-commit-notifier", move || {
                registry.notify(notified, seen)
            })
            .is_err()
            {
                state.callbacks.remove(&key);
                return Err(WorthQueryCommitWatchDenial::NotifierUnavailable);
            }
            state.notifier = Some(pulse);
        }
        drop(state);
        Ok(WorthQueryCommitWatchGuard {
            registry: Arc::clone(self),
            key,
        })
    }

    fn unregister(&self, key: u64) {
        let mut state = self.lock();
        state.callbacks.remove(&key);
        let notifier = state.notifier.clone();
        drop(state);
        if let Some(pulse) = notifier {
            pulse.wake_waiters();
        }
    }

    /// The notifier loop: wait for an advance, then call a snapshot of the
    /// callbacks outside the lock, each isolated from the others' panics.
    fn notify(&self, pulse: WorthQueryCommitPulse, mut seen: u64) {
        loop {
            let current = pulse.wait_after(seen, NOTIFIER_RECHECK);
            let callbacks = {
                let mut state = self.lock();
                if state.closed || state.callbacks.is_empty() {
                    state.notifier = None;
                    return;
                }
                if current == seen {
                    continue;
                }
                state.callbacks.values().cloned().collect::<Vec<_>>()
            };
            seen = current;
            for callback in callbacks {
                // A panicking callback stays registered and cannot stop the
                // notifier or the callbacks after it.
                drop(catch_unwind(AssertUnwindSafe(|| callback())));
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, RegistryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Closing the Home stops its notifier without waiting for it.
impl OwnedRuntimeHold for CommitWatchRegistry {
    fn release(&self) {
        let mut state = self.lock();
        state.closed = true;
        let notifier = state.notifier.clone();
        drop(state);
        if let Some(pulse) = notifier {
            pulse.wake_waiters();
        }
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryApplicationHome<Schema, Program> {
    /// This Home's committed-basis version: one atomic load, no query and no
    /// lock. Compare it with an earlier value to learn whether anything
    /// committed since your last read.
    pub fn committed_basis_version(&self) -> WorthQueryCommittedBasisVersion {
        WorthQueryCommittedBasisVersion::of(&self.primary_graph().commit_pulse())
    }

    /// Calls `on_commit` after every commit to this Home (any branch, any
    /// caller), on the Home's notifier thread, with no Host or Query lock held,
    /// until the returned guard drops or the Home closes.
    ///
    /// Commits close together may be reported by one call. A callback should
    /// return quickly (such as `egui::Context::request_repaint`); a panicking
    /// callback stays registered and does not affect the others. Never blocks
    /// a Home's close or retirement.
    pub fn on_commit(
        &self,
        on_commit: impl Fn() + Send + Sync + 'static,
    ) -> Result<WorthQueryCommitWatchGuard, WorthQueryCommitWatchDenial> {
        self.commit_watch.watch(
            &self.owned,
            self.primary_graph().commit_pulse(),
            Arc::new(on_commit),
        )
    }
}
