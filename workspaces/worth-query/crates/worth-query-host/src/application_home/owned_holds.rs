//! Owned handles (sessions and live subscriptions) that keep a Home's runtime
//! alive, and their release when the Home closes, so retirement never waits
//! on caller-side drop ordering.
//!
//! An owned session is thread-safe and is released from any thread. A live
//! subscription is pinned to the thread that opened it (its native lease is
//! thread-affine), so it registers in that thread's table and the Home
//! releases it when it closes on that thread, which is where a value holding
//! both the Home and the subscription necessarily closes it.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Weak as LocalWeak;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread::ThreadId;

/// A handle holding a clone of its Home's runtime until released.
pub(crate) trait OwnedRuntimeHold {
    /// Drops the runtime hold. Later use of the handle refuses as closed.
    fn release(&self);
}

enum HoldEntry {
    Shared(Weak<dyn OwnedRuntimeHold + Send + Sync>),
    Local {
        thread: ThreadId,
        key: u64,
        alive: Weak<()>,
    },
}

impl HoldEntry {
    fn is_alive(&self) -> bool {
        match self {
            Self::Shared(hold) => hold.strong_count() > 0,
            Self::Local { alive, .. } => alive.strong_count() > 0,
        }
    }
}

thread_local! {
    static LOCAL_HOLDS: RefCell<BTreeMap<u64, LocalWeak<dyn OwnedRuntimeHold>>> =
        const { RefCell::new(BTreeMap::new()) };
}
static NEXT_LOCAL_KEY: AtomicU64 = AtomicU64::new(0);

/// A thread-affine hold's registration; dropping it unregisters the hold.
pub(crate) struct LocalHoldRegistration {
    key: u64,
    _alive: Arc<()>,
}

impl Drop for LocalHoldRegistration {
    fn drop(&mut self) {
        // The registration never leaves its thread: it lives in a `!Send` hold.
        let _ = LOCAL_HOLDS.try_with(|holds| holds.borrow_mut().remove(&self.key));
    }
}

/// Every owned handle a Home has issued and not seen dropped.
#[derive(Default)]
pub(crate) struct OwnedRuntimeHolds {
    holds: Mutex<Vec<HoldEntry>>,
}

impl OwnedRuntimeHolds {
    /// Records a thread-safe `hold` so closing the Home releases it.
    pub(crate) fn register_shared(&self, hold: Weak<dyn OwnedRuntimeHold + Send + Sync>) {
        self.push(HoldEntry::Shared(hold));
    }

    /// Records a hold pinned to the current thread. Keep the returned
    /// registration inside the hold; dropping it unregisters the hold.
    pub(crate) fn register_local(
        &self,
        hold: LocalWeak<dyn OwnedRuntimeHold>,
    ) -> LocalHoldRegistration {
        let key = NEXT_LOCAL_KEY.fetch_add(1, Ordering::Relaxed);
        LOCAL_HOLDS.with(|holds| holds.borrow_mut().insert(key, hold));
        let alive = Arc::new(());
        self.push(HoldEntry::Local {
            thread: std::thread::current().id(),
            key,
            alive: Arc::downgrade(&alive),
        });
        LocalHoldRegistration { key, _alive: alive }
    }

    /// Dropped handles are pruned before the list grows (amortized O(1)).
    fn push(&self, entry: HoldEntry) {
        let mut holds = self
            .holds
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if holds.len() == holds.capacity() {
            holds.retain(HoldEntry::is_alive);
        }
        holds.push(entry);
    }

    /// Releases every live thread-safe hold and every live hold of the
    /// current thread, outside the registry lock. A hold pinned to another
    /// thread stays registered and keeps the runtime until it is dropped.
    pub(crate) fn release_all(&self) {
        let holds = std::mem::take(
            &mut *self
                .holds
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
        let current = std::thread::current().id();
        let mut elsewhere = Vec::new();
        for entry in holds {
            match entry {
                HoldEntry::Shared(hold) => {
                    if let Some(hold) = hold.upgrade() {
                        hold.release();
                    }
                }
                HoldEntry::Local { thread, key, .. } if thread == current => {
                    let hold = LOCAL_HOLDS.with(|holds| holds.borrow().get(&key).cloned());
                    if let Some(hold) = hold.and_then(|hold| hold.upgrade()) {
                        hold.release();
                    }
                }
                other @ HoldEntry::Local { .. } if other.is_alive() => elsewhere.push(other),
                HoldEntry::Local { .. } => {}
            }
        }
        if !elsewhere.is_empty() {
            self.holds
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .extend(elsewhere);
        }
    }
}
