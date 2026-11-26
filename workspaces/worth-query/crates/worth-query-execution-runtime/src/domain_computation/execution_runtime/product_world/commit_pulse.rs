//! A product root's committed-publication pulse: a version that advances after
//! a product advancement in which World answered a publication with an effect
//! has closed, and a bounded wait for the next advance.
//!
//! The pulse is a wake-up signal, never a record of a commit. Its only writer
//! is the product advancement scope below, fed where Query maps a World
//! publication outcome to an effect (immediate and durable alike). Readers
//! still select the branch head from World; a version only says "look again".
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// One product root's publication pulse. Clones share it.
#[derive(Clone)]
pub struct WorthQueryCommitPulse(Arc<PulseState>);

struct PulseState {
    incarnation: u64,
    published: AtomicU64,
    gate: Mutex<()>,
    changed: Condvar,
}

static NEXT_INCARNATION: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// Publications noted by the innermost open product advancement scope.
    static NOTED: Cell<Option<u64>> = const { Cell::new(None) };
}

impl WorthQueryCommitPulse {
    pub(crate) fn new() -> Self {
        Self(Arc::new(PulseState {
            incarnation: NEXT_INCARNATION.fetch_add(1, Ordering::Relaxed),
            published: AtomicU64::new(0),
            gate: Mutex::new(()),
            changed: Condvar::new(),
        }))
    }

    /// Distinguishes this root's versions from another root's, such as the
    /// same Home reopened. Unique within the process.
    pub fn incarnation(&self) -> u64 {
        self.0.incarnation
    }

    /// The current version: one atomic load, no lock.
    pub fn version(&self) -> u64 {
        self.0.published.load(Ordering::Acquire)
    }

    /// Blocks until the version differs from `seen`, [`Self::wake_waiters`]
    /// is called, or `timeout` elapses, then returns the current version.
    pub fn wait_after(&self, seen: u64, timeout: Duration) -> u64 {
        let gate = self
            .0
            .gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.version() != seen {
            return self.version();
        }
        drop(
            self.0
                .changed
                .wait_timeout(gate, timeout)
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        self.version()
    }

    /// Wakes every waiter without advancing, so a waiter can observe its own
    /// stop condition promptly.
    pub fn wake_waiters(&self) {
        drop(
            self.0
                .gate
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        self.0.changed.notify_all();
    }

    fn advance(&self) {
        let gate = self
            .0
            .gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.0.published.fetch_add(1, Ordering::AcqRel);
        drop(gate);
        self.0.changed.notify_all();
    }

    /// Opens the scope of one product advancement on this thread. Dropping it
    /// (also on unwind) advances the pulse once if a publication was noted.
    pub(crate) fn advancement_scope(&self) -> CommitPulseScope {
        CommitPulseScope {
            pulse: self.clone(),
            previous: NOTED.with(|noted| noted.replace(Some(0))),
            _thread: PhantomData,
        }
    }
}

/// Called where Query maps World's publication outcome to an effect: beside
/// the published-effect marker for immediate publications, and where a
/// durable (Directory) publication reports `Performed`. Outside a product
/// advancement scope (opening and bootstrap publications) there is nothing to
/// note.
pub fn note_publication() {
    NOTED.with(|noted| {
        if let Some(count) = noted.get() {
            noted.set(Some(count.saturating_add(1)));
        }
    });
}

/// One product advancement's publication scope, pinned to its thread.
#[must_use = "publications are noted only while the scope is held"]
pub(crate) struct CommitPulseScope {
    pulse: WorthQueryCommitPulse,
    previous: Option<u64>,
    _thread: PhantomData<Rc<()>>,
}

impl Drop for CommitPulseScope {
    fn drop(&mut self) {
        let noted = NOTED.with(|noted| noted.replace(self.previous.take()));
        if noted.is_some_and(|count| count > 0) {
            self.pulse.advance();
        }
    }
}
