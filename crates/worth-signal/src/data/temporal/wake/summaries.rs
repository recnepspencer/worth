use serde::{Deserialize, Serialize};

use super::{
    ScheduledTemporalWake, TemporalWakeId, TemporalWakeReschedule, TemporalWakeReuse, WakeOrdinal,
};

/// Diagnostic retention is bounded independently of active temporal readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalRetiredWakeCompactionReport {
    expired_now: u32,
    retained_width: u32,
    expired_total: u64,
    expired_digest: [u8; 32],
}

impl TemporalRetiredWakeCompactionReport {
    pub(crate) const fn new(
        expired_now: u32,
        retained_width: u32,
        expired_total: u64,
        expired_digest: [u8; 32],
    ) -> Self {
        Self {
            expired_now,
            retained_width,
            expired_total,
            expired_digest,
        }
    }

    pub const fn expired_now(self) -> u32 {
        self.expired_now
    }
    pub const fn retained_width(self) -> u32 {
        self.retained_width
    }
    pub const fn expired_total(self) -> u64 {
        self.expired_total
    }
    pub const fn expired_digest(self) -> [u8; 32] {
        self.expired_digest
    }
}

/// Cost-honest summary of runtime-owned temporal admission work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TemporalWakeAdmissionSummary {
    scheduled: Vec<ScheduledTemporalWake>,
    rescheduled: Vec<TemporalWakeReschedule>,
    reused: Vec<TemporalWakeReuse>,
}

impl TemporalWakeAdmissionSummary {
    pub(crate) fn record_scheduled(&mut self, wake: ScheduledTemporalWake) {
        self.scheduled.push(wake);
    }

    pub(crate) fn record_policy_supersession(&mut self, reschedule: TemporalWakeReschedule) {
        self.rescheduled.push(reschedule);
    }

    pub(crate) fn record_reschedule(&mut self, reschedule: TemporalWakeReschedule) {
        self.rescheduled.push(reschedule);
    }

    pub(crate) fn record_reused(&mut self, reuse: TemporalWakeReuse) {
        self.reused.push(reuse);
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.scheduled.extend(other.scheduled);
        self.rescheduled.extend(other.rescheduled);
        self.reused.extend(other.reused);
    }

    pub fn scheduled(&self) -> &[ScheduledTemporalWake] {
        &self.scheduled
    }

    pub fn rescheduled(&self) -> &[TemporalWakeReschedule] {
        &self.rescheduled
    }

    pub fn reused(&self) -> &[TemporalWakeReuse] {
        &self.reused
    }

    pub fn scheduled_count(&self) -> u64 {
        self.scheduled.len() as u64
    }

    pub fn rescheduled_count(&self) -> u64 {
        self.rescheduled.len() as u64
    }

    pub fn reused_count(&self) -> u64 {
        self.reused.len() as u64
    }

    pub fn total_decision_count(&self) -> u64 {
        self.scheduled_count()
            .saturating_add(self.rescheduled_count())
            .saturating_add(self.reused_count())
    }
}

/// Cost-honest summary of runtime-owned temporal wake state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TemporalWakeSummary {
    scheduled_count: u64,
    ready_count: u64,
    retired_count: u64,
    next_wake_id: TemporalWakeId,
    next_wake_ordinal: WakeOrdinal,
}

impl TemporalWakeSummary {
    pub(crate) fn new(
        scheduled_count: usize,
        ready_count: usize,
        retired_count: usize,
        next_wake_id: TemporalWakeId,
        next_wake_ordinal: WakeOrdinal,
    ) -> Self {
        Self {
            scheduled_count: scheduled_count as u64,
            ready_count: ready_count as u64,
            retired_count: retired_count as u64,
            next_wake_id,
            next_wake_ordinal,
        }
    }

    pub fn scheduled_count(self) -> u64 {
        self.scheduled_count
    }

    pub fn ready_count(self) -> u64 {
        self.ready_count
    }

    pub fn retired_count(self) -> u64 {
        self.retired_count
    }

    pub fn next_wake_id(self) -> TemporalWakeId {
        self.next_wake_id
    }

    pub fn next_wake_ordinal(self) -> WakeOrdinal {
        self.next_wake_ordinal
    }
}
