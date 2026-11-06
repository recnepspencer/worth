//! Optional receiver-owned storage observations; never admission or history authority.
use super::WorthQueryDocumentHistory;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Weak,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryHistoryEnrollmentReportForTest {
    /// Point accesses and visited range entries, including misses; excludes std BTree comparisons.
    pub node_probes: u64,
    pub attempt_probes: u64,
    pub custody_probes: u64,
    pub child_index_probes: u64,
    pub selection_accesses: u64,
    /// Actual entries visited while finding one oldest unprotected source.
    pub retention_frontier_probes: u64,
    /// Conservative owner work admitted independently of payload READ work.
    pub retention_quoted_work: u64,
    /// Conservative BTree path and label quotes from successful reservations.
    pub requested_backing_bytes: u64,
    /// The part of those quotes actually backed by retained execution tickets.
    /// Neither byte measure describes exact physical allocator traffic or resident memory.
    pub ticket_backing_bytes: u64,
}

#[derive(Default)]
struct Counters {
    accesses: [AtomicU64; 6],
    retention_work: AtomicU64,
    requested: AtomicU64,
    backed: AtomicU64,
}

pub(super) enum Access {
    Node,
    Attempt,
    Custody,
    Child,
    Selection,
    RetentionFrontier,
}
pub(super) struct EnrollmentSink(Weak<Counters>);

/// Dropping this observer ends recording; it retains no document or source lease.
pub struct WorthQueryHistoryEnrollmentObservationForTest {
    counters: Arc<Counters>,
}

impl WorthQueryHistoryEnrollmentObservationForTest {
    pub fn snapshot(&self) -> WorthQueryHistoryEnrollmentReportForTest {
        let access = |index: usize| self.counters.accesses[index].load(Ordering::Relaxed);
        WorthQueryHistoryEnrollmentReportForTest {
            node_probes: access(0),
            attempt_probes: access(1),
            custody_probes: access(2),
            child_index_probes: access(3),
            selection_accesses: access(4),
            retention_frontier_probes: access(5),
            retention_quoted_work: self.counters.retention_work.load(Ordering::Relaxed),
            requested_backing_bytes: self.counters.requested.load(Ordering::Relaxed),
            ticket_backing_bytes: self.counters.backed.load(Ordering::Relaxed),
        }
    }
    pub(super) fn new() -> (Self, EnrollmentSink) {
        let counters = Arc::new(Counters::default());
        let sink = EnrollmentSink(Arc::downgrade(&counters));
        (Self { counters }, sink)
    }
}

impl EnrollmentSink {
    pub(super) fn retention_work(&self, units: u64) {
        if let Some(counters) = self.0.upgrade() {
            counters.retention_work.fetch_add(units, Ordering::Relaxed);
        }
    }
    pub(super) fn access(&self, access: Access) {
        if let Some(counters) = self.0.upgrade() {
            counters.accesses[access as usize].fetch_add(1, Ordering::Relaxed);
        }
    }
    pub(super) fn backing(&self, requested: u64, backed: u64) {
        if let Some(counters) = self.0.upgrade() {
            counters.requested.fetch_add(requested, Ordering::Relaxed);
            counters.backed.fetch_add(backed, Ordering::Relaxed);
        }
    }
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    #[doc(hidden)]
    pub fn observe_history_enrollment_for_test(
        &mut self,
    ) -> WorthQueryHistoryEnrollmentObservationForTest {
        self.selection.lock().journal.observe()
    }
}
