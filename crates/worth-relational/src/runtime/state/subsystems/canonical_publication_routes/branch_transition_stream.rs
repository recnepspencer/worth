//! The runtime's branch transition stream: the sequence of performed fork,
//! archive and delete transitions, and the one place a transition is made
//! durable and cut over (Milestone 2 D8).
//!
//! A transition records under the routes' publication read lock, so a
//! checkpoint selection excludes it exactly as it excludes a commit, and
//! inside the patch-position reservation, so the commit head it follows is
//! fixed while it publishes and no commit can land between its durable step
//! and its cutover. Its sequence is its own: it never consumes a position.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::branch::{
    PositionedBranchTransition, RelationalBranchTransition, RelationalBranchTransitionDurableStep,
    RelationalBranchTransitionSequence,
};

use super::RelationalCanonicalPublicationRoutes;

#[derive(Debug, Default)]
pub(super) struct BranchTransitionStream {
    state: Mutex<BranchTransitionStreamState>,
}

#[derive(Debug, Default)]
struct BranchTransitionStreamState {
    /// The last sequence performed, or restored from a checkpoint's coverage.
    head: RelationalBranchTransitionSequence,
    /// Every transition this runtime performed or replayed, in sequence.
    log: Vec<Arc<PositionedBranchTransition>>,
}

/// Why a transition was not recorded. None of these left a live effect;
/// only an unresolved `Step` stop may have left a durable one.
#[derive(Debug)]
pub(crate) enum BranchTransitionRecordError<D, S> {
    /// The transition's preparation, re-checked inside the reservation,
    /// refused.
    Denied(D),
    /// The request stopped while it waited for the reservation.
    ReservationWaitStopped(worth_execution::WorkCeilingDenial),
    /// No sequence remains after the last performed one.
    SequenceCapacityExhausted,
    /// The durable step did not admit the cutover.
    Step(S),
}

impl RelationalCanonicalPublicationRoutes {
    /// Record one branch transition: durable first, then live.
    ///
    /// The transition waits for the reservation within `execution` in every
    /// runtime: its holder's cutover is bounded, and a lifecycle operation is
    /// never refused for ordinary concurrency.
    ///
    /// Inside the reservation, `prepare` re-checks what concurrent commits
    /// and transitions could have changed and yields the transition; the
    /// followed commit head and the next sequence are read; `step` makes the
    /// positioned transition durable; only then is the sequence advanced,
    /// the transition appended to the stream and `cutover` run. `cutover`
    /// must not fail: every fallible step belongs in `prepare` or before.
    ///
    /// Cost: one reservation wait, `prepare`, the durable step and
    /// `cutover`; O(1) stream bookkeeping.
    pub(crate) fn record_transition_with_cutover<D, R, S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        prepare: impl FnOnce() -> Result<RelationalBranchTransition, D>,
        cutover: impl FnOnce(&PositionedBranchTransition) -> R,
    ) -> Result<R, BranchTransitionRecordError<D, S::Stop>> {
        let _publication = self
            .lifecycle
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (_reservation, follows) = self
            .performed_stream
            .hold_reservation_within(execution)
            .map_err(BranchTransitionRecordError::ReservationWaitStopped)?;
        let transition = prepare().map_err(BranchTransitionRecordError::Denied)?;
        let sequence = self
            .branch_transitions
            .head()
            .next()
            .ok_or(BranchTransitionRecordError::SequenceCapacityExhausted)?;
        let positioned = PositionedBranchTransition::admit(sequence, follows, transition);
        step.take(&positioned)
            .map_err(BranchTransitionRecordError::Step)?;
        let positioned = Arc::new(positioned);
        self.branch_transitions.append(Arc::clone(&positioned));
        Ok(cutover(&positioned))
    }

    /// Empty routes that continue this runtime's branch transition stream,
    /// for the routes recovery rebuilds from its recovered inventory.
    pub(crate) fn continuing_branch_transitions(&self) -> Self {
        Self {
            branch_transitions: self.branch_transitions.share(),
            ..Self::default()
        }
    }

    /// Empty routes holding an independent copy of this runtime's branch
    /// transition stream, for a detached owner snapshot or runtime fork.
    pub(crate) fn detached_branch_transitions(&self) -> Self {
        Self {
            branch_transitions: self.branch_transitions.detached(),
            ..Self::default()
        }
    }

    /// The last branch transition sequence this runtime performed or
    /// restored. A checkpoint selected now covers exactly this sequence.
    pub(crate) fn branch_transition_head(&self) -> RelationalBranchTransitionSequence {
        self.branch_transitions.head()
    }

    /// Resume the stream from a checkpoint's retained transitions, which
    /// readmission proved contiguous from sequence one.
    pub(crate) fn restore_branch_transitions(&self, transitions: Vec<PositionedBranchTransition>) {
        let mut state = self.branch_transitions.state();
        if let Some(last) = transitions.last() {
            state.head = last.sequence();
        }
        state.log = transitions.into_iter().map(Arc::new).collect();
    }

    /// Every transition in the stream, in sequence.
    pub(crate) fn branch_transitions_snapshot(&self) -> Vec<Arc<PositionedBranchTransition>> {
        self.branch_transitions.state().log.clone()
    }
}

impl BranchTransitionStream {
    /// A stream continuing `self`, for routes a recovery rebuilt.
    pub(super) fn share(self: &Arc<Self>) -> Arc<Self> {
        Arc::clone(self)
    }

    /// An independent copy, for a runtime fork's speculative routes.
    pub(super) fn detached(&self) -> Arc<Self> {
        let state = self.state();
        Arc::new(Self {
            state: Mutex::new(BranchTransitionStreamState {
                head: state.head,
                log: state.log.clone(),
            }),
        })
    }

    fn head(&self) -> RelationalBranchTransitionSequence {
        self.state().head
    }

    fn append(&self, positioned: Arc<PositionedBranchTransition>) {
        let mut state = self.state();
        state.head = positioned.sequence();
        state.log.push(positioned);
    }

    fn state(&self) -> MutexGuard<'_, BranchTransitionStreamState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
