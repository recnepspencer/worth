//! Test-only rendezvous primitive behind the owner's rehearsal seams.
//!
//! A seam arms one point per owner. The single attempt that claims the point
//! reports the owner it is held for and waits, bounded, until the arming test
//! releases it; a point that resumes on its own is recorded as timed out, and
//! the guard that armed it fails by name on drop instead of on whatever
//! downstream assertion the unheld attempt happened to break. Every wait is
//! bounded so a released-nowhere test fails instead of hanging.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::identity::RuntimeWorldOwnerIdentity;

pub(crate) const OWNER_REHEARSAL_PAUSE_TIMEOUT: Duration = Duration::from_secs(5);

/// One seam's armed points, keyed by owner identity.
pub(crate) struct OwnerRehearsalRegistry {
    seam: &'static str,
    points: OnceLock<Mutex<HashMap<RuntimeWorldOwnerIdentity, Arc<OwnerRehearsalPoint>>>>,
}

impl OwnerRehearsalRegistry {
    pub(crate) const fn new(seam: &'static str) -> Self {
        Self {
            seam,
            points: OnceLock::new(),
        }
    }

    /// Arm one attempt on `owner` at this seam. The owner identity of the
    /// held attempt arrives on `reached`; dropping the returned guard releases
    /// the attempt and disarms the seam.
    pub(crate) fn arm(
        &'static self,
        owner: RuntimeWorldOwnerIdentity,
        reached: SyncSender<RuntimeWorldOwnerIdentity>,
    ) -> OwnerRehearsalGuard {
        let point = Arc::new(OwnerRehearsalPoint {
            armed: Mutex::new(true),
            reached,
            release: (Mutex::new(false), Condvar::new()),
            timed_out: AtomicBool::new(false),
        });
        let mut points = self
            .points()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        assert!(
            !points.contains_key(&owner),
            "only one {} rehearsal may be armed for an owner",
            self.seam
        );
        points.insert(owner, Arc::clone(&point));
        OwnerRehearsalGuard {
            registry: self,
            owner,
            point,
        }
    }

    /// Hold the single armed attempt on `owner` at this seam. Every other
    /// attempt passes straight through.
    pub(crate) fn pause(&self, owner: RuntimeWorldOwnerIdentity) {
        let Some(point) = self.armed(owner) else {
            return;
        };
        if point.claim() {
            point.wait_for_release(owner);
        }
    }

    fn armed(&self, owner: RuntimeWorldOwnerIdentity) -> Option<Arc<OwnerRehearsalPoint>> {
        self.points()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&owner)
            .map(Arc::clone)
    }

    fn disarm(&self, owner: RuntimeWorldOwnerIdentity, point: &Arc<OwnerRehearsalPoint>) {
        let mut points = self
            .points()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let owned_registration = points
            .get(&owner)
            .is_some_and(|current| Arc::ptr_eq(current, point));
        if owned_registration {
            points.remove(&owner);
        }
    }

    fn points(&self) -> &Mutex<HashMap<RuntimeWorldOwnerIdentity, Arc<OwnerRehearsalPoint>>> {
        self.points.get_or_init(|| Mutex::new(HashMap::new()))
    }
}

pub(crate) struct OwnerRehearsalGuard {
    registry: &'static OwnerRehearsalRegistry,
    owner: RuntimeWorldOwnerIdentity,
    point: Arc<OwnerRehearsalPoint>,
}

impl std::fmt::Debug for OwnerRehearsalGuard {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OwnerRehearsalGuard")
            .field("seam", &self.registry.seam)
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

impl Drop for OwnerRehearsalGuard {
    /// Release and disarm, then refuse to let a rehearsal that resumed itself
    /// pass as one this test held. The panicking check only avoids aborting
    /// the process on an unwind that is already reporting a failure.
    fn drop(&mut self) {
        self.point.release();
        self.registry.disarm(self.owner, &self.point);
        assert!(
            !self.point.timed_out() || std::thread::panicking(),
            "{} rehearsal was never released within {OWNER_REHEARSAL_PAUSE_TIMEOUT:?}",
            self.registry.seam
        );
    }
}

#[derive(Debug)]
struct OwnerRehearsalPoint {
    armed: Mutex<bool>,
    reached: SyncSender<RuntimeWorldOwnerIdentity>,
    release: (Mutex<bool>, Condvar),
    /// Set when the held attempt gave up waiting. A timed-out rehearsal
    /// resumed on its own, so everything the arming test observes afterwards
    /// describes a world it never actually controlled.
    timed_out: AtomicBool,
}

impl OwnerRehearsalPoint {
    fn claim(&self) -> bool {
        let mut armed = self.armed.lock().unwrap_or_else(|error| error.into_inner());
        let claimed = *armed;
        *armed = false;
        claimed
    }

    fn wait_for_release(&self, owner: RuntimeWorldOwnerIdentity) {
        if self.reached.send(owner).is_err() {
            return;
        }
        let deadline = Instant::now() + OWNER_REHEARSAL_PAUSE_TIMEOUT;
        let (opened, signal) = &self.release;
        let mut opened = opened.lock().unwrap_or_else(|error| error.into_inner());
        while !*opened {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                self.note_timeout();
                return;
            };
            let (next, result) = signal
                .wait_timeout(opened, remaining)
                .unwrap_or_else(|error| error.into_inner());
            opened = next;
            // A release that lands in the same instant the wait expires is a
            // release, not a timeout: the loop condition, not the clock, is
            // what decides whether this rehearsal was actually held.
            if result.timed_out() && !*opened {
                self.note_timeout();
                return;
            }
        }
    }

    fn note_timeout(&self) {
        self.timed_out.store(true, Ordering::SeqCst);
    }

    fn timed_out(&self) -> bool {
        self.timed_out.load(Ordering::SeqCst)
    }

    fn release(&self) {
        let (opened, signal) = &self.release;
        let mut opened = opened.lock().unwrap_or_else(|error| error.into_inner());
        *opened = true;
        signal.notify_all();
    }
}
