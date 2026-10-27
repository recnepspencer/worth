use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwapOption;

use crate::history::data::{CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;

use super::patch_position_reservation::{
    PatchPositionReservation, PatchPositionReservationAdmission, PatchPositionReservationGuard,
    RelationalPatchPositionReservationCounters,
};
use super::{CanonicalPublicationRecordError, CanonicalPublicationRoute};
use crate::durability::medium::RelationalDurableStepStop as DurableStepStop;

#[derive(Default)]
pub(super) struct PerformedPublicationStream {
    head: ArcSwapOption<PerformedPublicationNode>,
    index: Mutex<PerformedPublicationIndex>,
    recovered_position_floor: AtomicU64,
    reservation: PatchPositionReservation,
}

pub(super) struct PerformedPublicationNode {
    position: PatchStreamPosition,
    route: Arc<CanonicalPublicationRoute>,
    previous: Option<Arc<PerformedPublicationNode>>,
}

#[derive(Clone)]
pub(crate) struct PerformedCheckpointSelection {
    head: Option<Arc<PerformedPublicationNode>>,
    /// The last branch transition the selection covers (D8).
    covered_transitions: crate::branch::RelationalBranchTransitionSequence,
}

#[derive(Default)]
struct PerformedPublicationIndex {
    by_patch: BTreeMap<PatchStreamPosition, Arc<PerformedPublicationNode>>,
    by_commit: BTreeMap<CommitId, PatchStreamPosition>,
    indexed_head: Option<Arc<PerformedPublicationNode>>,
}

impl PerformedPublicationStream {
    pub(super) fn checkpoint_selection(
        &self,
        covered_transitions: crate::branch::RelationalBranchTransitionSequence,
    ) -> PerformedCheckpointSelection {
        PerformedCheckpointSelection {
            head: self.head.load_full(),
            covered_transitions,
        }
    }

    pub(super) fn first_unsettled_commit(&self) -> Option<CommitId> {
        let mut first = None;
        let mut cursor = self.head.load_full();
        while let Some(node) = cursor {
            if node.route.is_performed() && !node.route.is_settled() {
                let commit_id = node.route.commit.commit_id;
                first = Some(first.map_or(commit_id, |current: CommitId| current.min(commit_id)));
            }
            cursor = node.previous.clone();
        }
        first
    }

    pub(super) fn still_selects(&self, selection: &PerformedCheckpointSelection) -> bool {
        same_head(&self.head.load_full(), &selection.head)
    }

    /// Hold the reservation within `execution`, with the performed commit
    /// head a branch transition follows: fixed until the guard drops.
    pub(super) fn hold_reservation_within(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (PatchPositionReservationGuard<'_>, PatchStreamPosition),
        worth_execution::WorkCeilingDenial,
    > {
        let reservation = self.reservation.wait_within(execution)?;
        let head = self.performed_head_position(&self.head.load_full());
        Ok((reservation, PatchStreamPosition(head)))
    }

    /// The last assigned position: the performed head or the recovered floor,
    /// whichever is later.
    fn performed_head_position(&self, current: &Option<Arc<PerformedPublicationNode>>) -> u64 {
        current
            .as_ref()
            .map(|head| head.position.0)
            .unwrap_or(0)
            .max(self.recovered_position_floor.load(Ordering::Acquire))
    }

    pub(super) fn advance_position_floor(&self, floor: PatchStreamPosition) {
        self.recovered_position_floor
            .fetch_max(floor.0, Ordering::AcqRel);
    }

    pub(super) fn record_with_cutover(
        &self,
        route: Arc<CanonicalPublicationRoute>,
        resident: Arc<CanonicalCommitEnvelope>,
        admission: PatchPositionReservationAdmission<'_, '_, '_>,
        durable: impl FnOnce(
            &PositionedCanonicalCommit,
            &crate::branch::RelationalBranchRootDescriptor,
        ) -> Result<
            Option<crate::durability::medium::RelationalDurableCommitReadmission>,
            DurableStepStop,
        >,
        cutover: impl FnOnce(PatchStreamPosition),
    ) -> Result<Arc<PositionedCanonicalCommit>, CanonicalPublicationRecordError> {
        let _reservation = self.reservation.enter(admission)?;
        let current = self.head.load_full();
        let prior_position = self.performed_head_position(&current);
        let Some(next_position) = prior_position.checked_add(1) else {
            self.reservation.overflows.fetch_add(1, Ordering::Relaxed);
            return Err(CanonicalPublicationRecordError::PositionCapacityExhausted);
        };
        let positioned = Arc::new(PositionedCanonicalCommit::admit(
            super::CanonicalPositionAdmission::performed(
                PatchStreamPosition(next_position),
                resident,
            ),
        ));
        // Durable before live: the position is assigned only once the medium
        // holds it, and the cutover below runs only after that.
        let journal =
            durable(&positioned, &route.roots).map_err(CanonicalPublicationRecordError::Durable)?;
        route
            .envelope
            .bind_performed(&positioned, journal.map(|receipt| receipt.into_parts()));
        let node = Arc::new(PerformedPublicationNode {
            position: positioned.position(),
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
            .enter(PatchPositionReservationAdmission::DeferWhenHeld)
            .map_err(|_| "recovered canonical stream reservation is unexpectedly contended")?;
        let current = self.head.load_full();
        if current
            .as_ref()
            .is_some_and(|head| head.position >= positioned.position())
        {
            return Err("recovered canonical stream order is not strictly increasing");
        }
        let node = Arc::new(PerformedPublicationNode {
            position: positioned.position(),
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
            .and_then(|node| node.route.envelope.get())
    }

    pub(super) fn positioned_at(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.refresh_index()
            .by_patch
            .get(&position)
            .filter(|node| node.route.is_visible())
            .and_then(|node| node.route.envelope.positioned_if_resident().map(Arc::new))
    }

    pub(super) fn visible_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.refresh_index()
            .by_patch
            .values()
            .filter(|node| node.route.is_visible())
            .filter_map(|node| node.route.envelope.get())
            .collect()
    }

    pub(super) fn latest(&self) -> Option<(PatchStreamPosition, CommitId)> {
        self.refresh_index()
            .by_patch
            .iter()
            .rev()
            .find(|(_, node)| node.route.is_visible())
            .map(|(position, node)| (*position, node.route.commit.commit_id))
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
            .map(|(position, node)| (*position, node.route.commit.commit_id))
            .collect()
    }

    pub(super) fn latest_references(&self, limit: usize) -> Vec<(PatchStreamPosition, CommitId)> {
        self.refresh_index()
            .by_patch
            .iter()
            .rev()
            .filter(|(_, node)| node.route.is_visible())
            .take(limit)
            .map(|(position, node)| (*position, node.route.commit.commit_id))
            .collect()
    }

    pub(super) fn positioned(&self, commit_id: CommitId) -> Option<Arc<PositionedCanonicalCommit>> {
        let index = self.refresh_index();
        index
            .by_commit
            .get(&commit_id)
            .and_then(|position| index.by_patch.get(position))
            .filter(|node| node.route.is_visible())
            .and_then(|node| node.route.envelope.positioned_if_resident().map(Arc::new))
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
            let position = node.position;
            index
                .by_commit
                .insert(node.route.commit.commit_id, position);
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
            .filter_map(|node| node.route.envelope.positioned_if_resident().map(Arc::new))
            .collect()
    }

    pub(super) fn reservation_counters(&self) -> RelationalPatchPositionReservationCounters {
        self.reservation.counters()
    }
}

impl PerformedCheckpointSelection {
    pub(crate) fn latest_receipt(&self) -> Option<crate::history::data::RelationalCommitReceipt> {
        self.head
            .as_ref()
            .filter(|head| head.route.is_performed())
            .map(|head| head.route.commit.clone())
    }
    /// The last branch transition sequence the checkpoint covers.
    pub(crate) fn covered_transitions(&self) -> crate::branch::RelationalBranchTransitionSequence {
        self.covered_transitions
    }

    pub(crate) fn positioned_snapshot(&self) -> Vec<PositionedCanonicalCommit> {
        let mut positioned = Vec::new();
        let mut cursor = self.head.clone();
        while let Some(node) = cursor {
            if node.route.is_performed()
                && let Some(resident) = node.route.envelope.positioned_if_resident()
            {
                positioned.push(resident);
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
