use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwapOption;

use crate::history::data::{CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;

use super::CanonicalPublicationRoute;

#[derive(Default)]
pub(super) struct PerformedPublicationStream {
    head: ArcSwapOption<PerformedPublicationNode>,
    index: Mutex<PerformedPublicationIndex>,
    recovered_position_floor: AtomicU64,
    reservation: PatchPositionReservation,
}

pub(super) struct PerformedPublicationNode {
    positioned: Arc<PositionedCanonicalCommit>,
    route: Arc<CanonicalPublicationRoute>,
    previous: Option<Arc<PerformedPublicationNode>>,
}

#[derive(Clone)]
pub(crate) struct PerformedCheckpointSelection {
    head: Option<Arc<PerformedPublicationNode>>,
}

#[derive(Default)]
struct PerformedPublicationIndex {
    by_patch: BTreeMap<PatchStreamPosition, Arc<PerformedPublicationNode>>,
    by_commit: BTreeMap<CommitId, PatchStreamPosition>,
    indexed_head: Option<Arc<PerformedPublicationNode>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelationalPatchPositionReservationCounters {
    pub contacts: u64,
    pub deferrals: u64,
    pub assignments: u64,
    pub overflows: u64,
}

pub(super) enum PerformedPublicationRecordError {
    ReservationContended,
    PositionCapacityExhausted,
}

#[derive(Default)]
struct PatchPositionReservation {
    contacts: AtomicU64,
    deferrals: AtomicU64,
    assignments: AtomicU64,
    overflows: AtomicU64,
    active: AtomicBool,
}

struct PatchPositionReservationGuard<'reservation> {
    reservation: &'reservation PatchPositionReservation,
}

impl PerformedPublicationStream {
    pub(super) fn checkpoint_selection(&self) -> PerformedCheckpointSelection {
        PerformedCheckpointSelection {
            head: self.head.load_full(),
        }
    }

    pub(super) fn first_unsettled_commit(&self) -> Option<CommitId> {
        let mut first = None;
        let mut cursor = self.head.load_full();
        while let Some(node) = cursor {
            if node.route.is_performed() && !node.route.is_settled() {
                let commit_id = node.positioned.envelope().commit.commit_id;
                first = Some(first.map_or(commit_id, |current: CommitId| current.min(commit_id)));
            }
            cursor = node.previous.clone();
        }
        first
    }

    pub(super) fn still_selects(&self, selection: &PerformedCheckpointSelection) -> bool {
        same_head(&self.head.load_full(), &selection.head)
    }

    pub(super) fn advance_position_floor(&self, floor: PatchStreamPosition) {
        self.recovered_position_floor
            .fetch_max(floor.0, Ordering::AcqRel);
    }

    pub(super) fn record_with_cutover(
        &self,
        route: Arc<CanonicalPublicationRoute>,
        cutover: impl FnOnce(PatchStreamPosition),
    ) -> Result<Arc<PositionedCanonicalCommit>, PerformedPublicationRecordError> {
        let _reservation = self.reservation.try_enter()?;
        let current = self.head.load_full();
        let prior_position = current
            .as_ref()
            .map(|head| head.positioned.position().0)
            .unwrap_or(0)
            .max(self.recovered_position_floor.load(Ordering::Acquire));
        let Some(next_position) = prior_position.checked_add(1) else {
            self.reservation.overflows.fetch_add(1, Ordering::Relaxed);
            return Err(PerformedPublicationRecordError::PositionCapacityExhausted);
        };
        let positioned = Arc::new(PositionedCanonicalCommit::admit(
            super::CanonicalPositionAdmission::performed(
                PatchStreamPosition(next_position),
                Arc::clone(&route.envelope),
            ),
        ));
        let node = Arc::new(PerformedPublicationNode {
            positioned: Arc::clone(&positioned),
            route: Arc::clone(&route),
            previous: current,
        });
        route.set_position(PatchStreamPosition(next_position));
        self.head.store(Some(node));
        self.reservation.assignments.fetch_add(1, Ordering::Relaxed);
        cutover(PatchStreamPosition(next_position));
        route.mark_performed();
        Ok(positioned)
    }

    pub(super) fn link_recovered(
        &self,
        positioned: Arc<PositionedCanonicalCommit>,
        route: Arc<CanonicalPublicationRoute>,
    ) -> Result<(), &'static str> {
        let _reservation = self
            .reservation
            .try_enter()
            .map_err(|_| "recovered canonical stream reservation is unexpectedly contended")?;
        let current = self.head.load_full();
        if current
            .as_ref()
            .is_some_and(|head| head.positioned.position() >= positioned.position())
        {
            return Err("recovered canonical stream order is not strictly increasing");
        }
        let node = Arc::new(PerformedPublicationNode {
            positioned: Arc::clone(&positioned),
            route,
            previous: current,
        });
        self.head.store(Some(node));
        self.advance_position_floor(positioned.position());
        Ok(())
    }

    pub(super) fn envelope_at(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.refresh_index()
            .by_patch
            .get(&position)
            .filter(|node| node.route.is_visible())
            .map(|node| Arc::clone(node.positioned.canonical_arc()))
    }

    pub(super) fn positioned_at(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.refresh_index()
            .by_patch
            .get(&position)
            .filter(|node| node.route.is_visible())
            .map(|node| Arc::clone(&node.positioned))
    }

    pub(super) fn visible_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.refresh_index()
            .by_patch
            .values()
            .filter(|node| node.route.is_visible())
            .map(|node| Arc::clone(node.positioned.canonical_arc()))
            .collect()
    }

    pub(super) fn latest(&self) -> Option<(PatchStreamPosition, CommitId)> {
        self.refresh_index()
            .by_patch
            .iter()
            .rev()
            .find(|(_, node)| node.route.is_visible())
            .map(|(position, node)| (*position, node.positioned.envelope().commit.commit_id))
    }

    pub(super) fn after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        let start = after_position
            .map(std::ops::Bound::Excluded)
            .unwrap_or(std::ops::Bound::Unbounded);
        self.refresh_index()
            .by_patch
            .range((start, std::ops::Bound::Unbounded))
            .filter(|(_, node)| node.route.is_visible())
            .take(max_commits)
            .map(|(position, node)| (*position, node.positioned.envelope().commit.commit_id))
            .collect()
    }

    pub(super) fn positioned(&self, commit_id: CommitId) -> Option<Arc<PositionedCanonicalCommit>> {
        let index = self.refresh_index();
        index
            .by_commit
            .get(&commit_id)
            .and_then(|position| index.by_patch.get(position))
            .filter(|node| node.route.is_visible())
            .map(|node| Arc::clone(&node.positioned))
    }

    fn refresh_index(&self) -> std::sync::MutexGuard<'_, PerformedPublicationIndex> {
        let mut index = self
            .index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let head = self.head.load_full();
        let mut pending = Vec::new();
        let mut cursor = head.clone();
        while let Some(node) = cursor {
            if index
                .indexed_head
                .as_ref()
                .is_some_and(|indexed| Arc::ptr_eq(indexed, &node))
            {
                break;
            }
            cursor = node.previous.clone();
            pending.push(node);
        }
        for node in pending.into_iter().rev() {
            let position = node.positioned.position();
            index
                .by_commit
                .insert(node.route.envelope.commit.commit_id, position);
            index.by_patch.insert(position, Arc::clone(&node));
        }
        index.indexed_head = head;
        index
    }

    pub(super) fn positioned_snapshot(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.refresh_index()
            .by_patch
            .values()
            .filter(|node| node.route.is_performed())
            .map(|node| Arc::clone(&node.positioned))
            .collect()
    }

    pub(super) fn reservation_counters(&self) -> RelationalPatchPositionReservationCounters {
        self.reservation.counters()
    }
}

impl PerformedCheckpointSelection {
    pub(crate) fn positioned_snapshot(&self) -> Vec<PositionedCanonicalCommit> {
        let mut positioned = Vec::new();
        let mut cursor = self.head.clone();
        while let Some(node) = cursor {
            if node.route.is_performed() {
                positioned.push(node.positioned.as_ref().clone());
            }
            cursor = node.previous.clone();
        }
        positioned.reverse();
        positioned
    }
}

fn same_head(
    left: &Option<Arc<PerformedPublicationNode>>,
    right: &Option<Arc<PerformedPublicationNode>>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

impl PatchPositionReservation {
    fn try_enter(
        &self,
    ) -> Result<PatchPositionReservationGuard<'_>, PerformedPublicationRecordError> {
        self.contacts.fetch_add(1, Ordering::Relaxed);
        if self
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            self.deferrals.fetch_add(1, Ordering::Relaxed);
            return Err(PerformedPublicationRecordError::ReservationContended);
        }
        Ok(PatchPositionReservationGuard { reservation: self })
    }

    fn counters(&self) -> RelationalPatchPositionReservationCounters {
        RelationalPatchPositionReservationCounters {
            contacts: self.contacts.load(Ordering::Relaxed),
            deferrals: self.deferrals.load(Ordering::Relaxed),
            assignments: self.assignments.load(Ordering::Relaxed),
            overflows: self.overflows.load(Ordering::Relaxed),
        }
    }
}

impl Drop for PatchPositionReservationGuard<'_> {
    fn drop(&mut self) {
        self.reservation.active.store(false, Ordering::Release);
    }
}
