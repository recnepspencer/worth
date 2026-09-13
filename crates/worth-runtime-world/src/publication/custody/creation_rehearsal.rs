//! Test-only rehearsal control for the forked-branch product-unpublished
//! route.
//!
//! Finalization only reaches that route when an owner-issued authority is
//! withheld after the destination commit is already installed. Every such
//! denial is worst-case reserved capacity, so no honest fixture can starve one
//! deterministically. This control withholds exactly the observation authority
//! `issue_observation_authority` already models as a typed admission denial, then holds the
//! diverted attempt at the recovery-record construction boundary so another
//! thread can read the close-admission ledger while the operation reservation
//! is the only custody standing between recovery and `close()`.
//!
//! The rehearsal is keyed by owner identity and armed for one attempt, never
//! process-global, and every wait is bounded so a released-nowhere test fails
//! by name instead of hanging.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::identity::{ProductUnpublishedOwnerEffectsIdentity, RuntimeWorldOwnerIdentity};
use crate::lifecycle::RuntimeWorldOwnerRoot;

const FORKED_FINALIZATION_PAUSE_TIMEOUT: Duration = Duration::from_secs(5);

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Arm one forked finalization on this owner to take its product-unpublished
    /// route and stop at the recovery-record boundary. The identity of the
    /// paused attempt arrives on `reached`; dropping the returned guard releases
    /// the attempt and disarms the rehearsal.
    pub(crate) fn rehearse_forked_finalization_recovery(
        &self,
        reached: SyncSender<ProductUnpublishedOwnerEffectsIdentity>,
    ) -> ForkedFinalizationRehearsalGuard {
        let owner = self.owner_identity();
        let rehearsal = Arc::new(ForkedFinalizationRehearsal {
            observation_authority_armed: Mutex::new(true),
            reached,
            release: (Mutex::new(false), Condvar::new()),
            timed_out: AtomicBool::new(false),
        });
        let mut armed = rehearsals()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        assert!(
            !armed.contains_key(&owner),
            "only one forked finalization rehearsal may be armed for an owner"
        );
        armed.insert(owner, Arc::clone(&rehearsal));
        ForkedFinalizationRehearsalGuard { owner, rehearsal }
    }
}

/// Withhold the owner-issued observation authority of the single rehearsed
/// attempt. Every other attempt keeps the authority its owner issued.
pub(super) fn withhold_observation_authority_under_rehearsal<A>(
    identity: &ProductUnpublishedOwnerEffectsIdentity,
    authority: Result<A, crate::recovery::ProductUnpublishedCause>,
) -> Result<A, crate::recovery::ProductUnpublishedCause> {
    let Some(rehearsal) = armed_rehearsal(identity) else {
        return authority;
    };
    if !rehearsal.claim_observation_authority() {
        return authority;
    }
    drop(authority);
    Err(crate::recovery::ProductUnpublishedCause::DestinationAdmissionDenied)
}

/// Hold the rehearsed attempt after `begin_recovery` and before its record is
/// installed. This is the single construction boundary every product-unpublished
/// forked-branch record passes through.
pub(super) fn pause_before_forked_recovery_record(
    identity: &ProductUnpublishedOwnerEffectsIdentity,
) {
    if let Some(rehearsal) = armed_rehearsal(identity) {
        rehearsal.wait_for_release(identity.clone());
    }
}

pub(crate) struct ForkedFinalizationRehearsalGuard {
    owner: RuntimeWorldOwnerIdentity,
    rehearsal: Arc<ForkedFinalizationRehearsal>,
}

impl std::fmt::Debug for ForkedFinalizationRehearsalGuard {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ForkedFinalizationRehearsalGuard")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

impl Drop for ForkedFinalizationRehearsalGuard {
    fn drop(&mut self) {
        self.rehearsal.release();
        let mut armed = rehearsals()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let owned_registration = armed
            .get(&self.owner)
            .is_some_and(|current| Arc::ptr_eq(current, &self.rehearsal));
        if owned_registration {
            armed.remove(&self.owner);
        }
        drop(armed);
        // A rehearsal that resumed on its own describes a world the arming
        // test never controlled, so the test fails here by name instead of on
        // whatever downstream assertion the unheld attempt happened to break.
        assert!(
            !self.rehearsal.timed_out() || std::thread::panicking(),
            "forked finalization rehearsal was never released within {FORKED_FINALIZATION_PAUSE_TIMEOUT:?}"
        );
    }
}

#[derive(Debug)]
struct ForkedFinalizationRehearsal {
    observation_authority_armed: Mutex<bool>,
    reached: SyncSender<ProductUnpublishedOwnerEffectsIdentity>,
    release: (Mutex<bool>, Condvar),
    /// Set when the held attempt gave up waiting and resumed on its own.
    timed_out: AtomicBool,
}

impl ForkedFinalizationRehearsal {
    fn claim_observation_authority(&self) -> bool {
        let mut armed = self
            .observation_authority_armed
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let claimed = *armed;
        *armed = false;
        claimed
    }

    fn wait_for_release(&self, identity: ProductUnpublishedOwnerEffectsIdentity) {
        if self.reached.send(identity).is_err() {
            return;
        }
        let deadline = Instant::now() + FORKED_FINALIZATION_PAUSE_TIMEOUT;
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
            // release, not a timeout: the loop condition decides.
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

static ARMED_REHEARSALS: OnceLock<
    Mutex<HashMap<RuntimeWorldOwnerIdentity, Arc<ForkedFinalizationRehearsal>>>,
> = OnceLock::new();

fn rehearsals(
) -> &'static Mutex<HashMap<RuntimeWorldOwnerIdentity, Arc<ForkedFinalizationRehearsal>>> {
    ARMED_REHEARSALS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn armed_rehearsal(
    identity: &ProductUnpublishedOwnerEffectsIdentity,
) -> Option<Arc<ForkedFinalizationRehearsal>> {
    rehearsals()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&identity.owner_identity())
        .map(Arc::clone)
}
