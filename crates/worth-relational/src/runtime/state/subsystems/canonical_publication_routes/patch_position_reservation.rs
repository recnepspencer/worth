//! The one global patch-position reservation: whoever holds it assigns the
//! next canonical position, and with a medium attached makes that position
//! durable, before the live cutover.
//!
//! How a publisher meets a held reservation is the runtime's admission:
//!
//! - Memory-only runtimes defer. The holder only links an in-memory node, so a
//!   contended publisher returns `PatchPositionReservationContended` and the
//!   caller decides whether to retry.
//! - Runtimes with a durable medium wait (D2). The holder is doing durable I/O
//!   for its own commit; deferring would turn ordinary concurrency into
//!   caller-visible refusals. Waiters are served first come, first served,
//!   within the caller's execution deadline, and observe cancellation.
//! - Branch transitions wait in either runtime (D8): a fork, archive or
//!   delete is not deferred because a commit is linking its node.
//!
//! Lock order: a publisher takes its own branch's critical section and
//! publication state, then this reservation. The holder never takes another
//! branch's critical section or state while holding it, and its durable step
//! and cutover touch only its own branch, so a waiter that holds its branch
//! cannot be in the holder's way. Two publishers on one branch meet at that
//! branch's critical section, before either reaches the reservation.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Duration;

use super::CanonicalPublicationRecordError;

/// Longest a waiter sleeps between checks of its request's cancellation and
/// deadline. Release wakes waiters at once; this bounds only how late a
/// cancelled or expired waiter notices, since cancellation has no wake.
const WAITER_CANCELLATION_CHECK: Duration = Duration::from_millis(2);

/// How a publication meets a reservation another publisher holds.
#[derive(Clone, Copy)]
pub(crate) enum PatchPositionReservationAdmission<'request, 'scope, 'authority> {
    /// Return `ReservationContended` at once (memory-only runtimes).
    DeferWhenHeld,
    /// Wait in arrival order until the reservation is free, or until the
    /// request is cancelled or its deadline passes (durable runtimes).
    WaitWithin(&'request worth_execution::ExecutionRequest<'scope, 'authority>),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelationalPatchPositionReservationCounters {
    pub contacts: u64,
    pub deferrals: u64,
    pub assignments: u64,
    pub overflows: u64,
}

#[derive(Default)]
pub(super) struct PatchPositionReservation {
    contacts: AtomicU64,
    deferrals: AtomicU64,
    pub(super) assignments: AtomicU64,
    pub(super) overflows: AtomicU64,
    queue: Mutex<ReservationQueue>,
    released: Condvar,
}

#[derive(Default)]
struct ReservationQueue {
    held: bool,
    next_ticket: u64,
    /// Tickets of waiting publishers, oldest first.
    waiting: VecDeque<u64>,
}

pub(super) struct PatchPositionReservationGuard<'reservation> {
    reservation: &'reservation PatchPositionReservation,
}

impl PatchPositionReservation {
    /// Take the reservation as `admission` says.
    ///
    /// Cost: O(1) when free; a waiter wakes once per release and at most
    /// every `WAITER_CANCELLATION_CHECK`, and removing a stopped waiter is
    /// O(waiters).
    pub(super) fn enter(
        &self,
        admission: PatchPositionReservationAdmission<'_, '_, '_>,
    ) -> Result<PatchPositionReservationGuard<'_>, CanonicalPublicationRecordError> {
        match admission {
            PatchPositionReservationAdmission::DeferWhenHeld => {
                self.contacts.fetch_add(1, Ordering::Relaxed);
                let mut queue = self.queue();
                if !queue.held && queue.waiting.is_empty() {
                    queue.held = true;
                    return Ok(PatchPositionReservationGuard { reservation: self });
                }
                self.deferrals.fetch_add(1, Ordering::Relaxed);
                Err(CanonicalPublicationRecordError::ReservationContended)
            }
            PatchPositionReservationAdmission::WaitWithin(execution) => self
                .wait_within(execution)
                .map_err(CanonicalPublicationRecordError::ReservationWaitStopped),
        }
    }

    /// Take the reservation in arrival order, waiting behind the holder and
    /// every earlier waiter within `execution`'s cancellation and deadline. A
    /// stopped request leaves the queue before any effect: it was assigned
    /// nothing. Branch transitions always take the reservation this way.
    pub(super) fn wait_within(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PatchPositionReservationGuard<'_>, worth_execution::WorkCeilingDenial> {
        self.contacts.fetch_add(1, Ordering::Relaxed);
        let mut queue = self.queue();
        if !queue.held && queue.waiting.is_empty() {
            queue.held = true;
            return Ok(PatchPositionReservationGuard { reservation: self });
        }
        let ticket = queue.next_ticket;
        queue.next_ticket = queue.next_ticket.wrapping_add(1);
        queue.waiting.push_back(ticket);
        loop {
            if !queue.held && queue.waiting.front() == Some(&ticket) {
                queue.waiting.pop_front();
                queue.held = true;
                return Ok(PatchPositionReservationGuard { reservation: self });
            }
            if let Err(stop) = execution.consult() {
                queue.waiting.retain(|waiting| *waiting != ticket);
                drop(queue);
                self.released.notify_all();
                return Err(stop);
            }
            queue = self
                .released
                .wait_timeout(queue, WAITER_CANCELLATION_CHECK)
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
    }

    pub(super) fn counters(&self) -> RelationalPatchPositionReservationCounters {
        RelationalPatchPositionReservationCounters {
            contacts: self.contacts.load(Ordering::Relaxed),
            deferrals: self.deferrals.load(Ordering::Relaxed),
            assignments: self.assignments.load(Ordering::Relaxed),
            overflows: self.overflows.load(Ordering::Relaxed),
        }
    }

    fn queue(&self) -> MutexGuard<'_, ReservationQueue> {
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for PatchPositionReservationGuard<'_> {
    fn drop(&mut self) {
        self.reservation.queue().held = false;
        self.reservation.released.notify_all();
    }
}
