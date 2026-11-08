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
    RelationalBranchTransitionSequence, RelationalTransitionPublication,
};

use super::RelationalCanonicalPublicationRoutes;

#[path = "branch_transition_stream/retained_log.rs"]
mod retained_log;
#[path = "branch_transition_stream/schema_inputs.rs"]
mod schema_inputs;
#[path = "branch_transition_stream/source_recovery.rs"]
mod source_recovery;
pub(crate) use schema_inputs::{
    OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial, OriginalEmptySchemaInstallation,
    OriginalEmptySchemaObservation, RetainedBranchTransition,
};

#[derive(Debug, Default)]
pub(super) struct BranchTransitionStream {
    state: Mutex<BranchTransitionStreamState>,
}

#[derive(Debug, Default)]
struct BranchTransitionStreamState {
    /// The last sequence performed, or restored from a checkpoint's coverage.
    head: RelationalBranchTransitionSequence,
    /// Last released prefix; zero while memory-only history remains retained.
    covered: RelationalBranchTransitionSequence,
    // Coverage controls the ordinary suffix, not the lifetime of originals.
    log: Vec<RetainedBranchTransition>,
    birth: Option<OriginalEmptySchemaObservation>,
    installation: Option<OriginalEmptySchemaInstallation>,
    initial_inputs:
        Option<Arc<crate::durability::source_initial_inputs::image::InitialInputsImage>>,
    home_world_selection: Option<
        Arc<
            worth_proof::VerifiedHomeWorldSelection<
                Arc<crate::durability::RelationalWorldSourceCatalog>,
            >,
        >,
    >,
    _log_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Why a transition was not recorded. None of these left a live effect;
/// only an unresolved `Step` stop may have left a durable one.
#[derive(Debug)]
pub(crate) enum BranchTransitionRecordError<D, S> {
    /// The transition's preparation, re-checked inside the reservation,
    /// refused.
    Denied(D),
    /// The original request stopped during reservation or storage preparation.
    ReservationWaitStopped(worth_execution::WorkCeilingDenial),
    /// No sequence remains after the last performed one.
    SequenceCapacityExhausted,
    /// The actual retained transition backing cannot be represented.
    StorageCapacityExhausted,
    /// A restored sequence disagrees with this stream's original prefix.
    OriginalPrefixMismatch,
    OriginalInputsStopped(crate::durability::data::RelationalRecoveryStop),
    /// The durable step did not admit the cutover.
    Step(S),
}

/// Lifecycle admission acquired before any live branch coordination or state.
pub(crate) struct BranchTransitionPublication<'routes> {
    routes: &'routes RelationalCanonicalPublicationRoutes,
    _lifecycle: std::sync::RwLockReadGuard<'routes, ()>,
}

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn try_initial_schema_publication(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, ()>, ()> {
        self.lifecycle.try_write().map_err(|_| ())
    }
    pub(crate) fn begin_branch_transition(&self) -> BranchTransitionPublication<'_> {
        #[cfg(test)]
        self.lifecycle_order.transition_contact();
        BranchTransitionPublication {
            routes: self,
            _lifecycle: self
                .lifecycle
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }
}

impl BranchTransitionPublication<'_> {
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
    pub(crate) fn record_with_cutover<D, R, S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        schema: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
        prepare: impl FnOnce() -> Result<RelationalBranchTransition, D>,
        cutover: impl FnOnce(&PositionedBranchTransition) -> R,
    ) -> Result<R, BranchTransitionRecordError<D, S::Stop>> {
        let (_reservation, follows) = self
            .routes
            .performed_stream
            .hold_reservation_within(execution)
            .map_err(BranchTransitionRecordError::ReservationWaitStopped)?;
        let transition = prepare().map_err(BranchTransitionRecordError::Denied)?;
        let sequence = self
            .routes
            .branch_transitions
            .head()
            .next()
            .ok_or(BranchTransitionRecordError::SequenceCapacityExhausted)?;
        let carrier = schema
            .recorded_image(execution)
            .map_err(BranchTransitionRecordError::OriginalInputsStopped)?
            .carrier_digest();
        let positioned = PositionedBranchTransition::admit(sequence, follows, transition)
            .with_schema_carrier(Some(carrier));
        let retained = self
            .routes
            .branch_transitions
            .prepare_append(execution, &positioned)
            .map_err(|denial| match denial {
                OriginalBranchSchemaInputsDenial::SizeOverflow => {
                    BranchTransitionRecordError::StorageCapacityExhausted
                }
                OriginalBranchSchemaInputsDenial::Request(cause) => {
                    BranchTransitionRecordError::ReservationWaitStopped(cause)
                }
                OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch => {
                    BranchTransitionRecordError::OriginalPrefixMismatch
                }
            })?;
        let publication: RelationalTransitionPublication = step
            .take(positioned)
            .map_err(BranchTransitionRecordError::Step)?;
        Ok(publication.cutover(|positioned| {
            let positioned = self
                .routes
                .branch_transitions
                .append(positioned, schema, retained);
            cutover(&positioned)
        }))
    }
}

impl RelationalCanonicalPublicationRoutes {
    /// Empty routes that continue this runtime's branch transition stream,
    /// for the routes recovery rebuilds from its recovered inventory.
    pub(crate) fn continuing_branch_transitions(&self) -> Self {
        Self {
            checkpoint_catalog: std::sync::RwLock::new(
                self.checkpoint_catalog
                    .read()
                    .unwrap_or_else(|error| error.into_inner())
                    .clone(),
            ),
            branch_transitions: self.branch_transitions.share(),
            ..Self::default()
        }
    }

    /// Empty routes holding an independent copy of this runtime's branch
    /// transition stream, for a detached owner snapshot or runtime fork.
    pub(crate) fn detached_branch_transitions(&self) -> Self {
        Self {
            checkpoint_catalog: std::sync::RwLock::new(
                self.checkpoint_catalog
                    .read()
                    .unwrap_or_else(|error| error.into_inner())
                    .clone(),
            ),
            branch_transitions: self.branch_transitions.detached(),
            ..Self::default()
        }
    }

    /// The last branch transition sequence this runtime performed or
    /// restored. A checkpoint selected now covers exactly this sequence.
    pub(crate) fn branch_transition_head(&self) -> RelationalBranchTransitionSequence {
        self.branch_transitions.head()
    }

    pub(crate) fn restore_branch_transition_coverage(
        &self,
        covered: RelationalBranchTransitionSequence,
    ) {
        let mut state = self.branch_transitions.state();
        state.head = covered;
        state.covered = covered;
        // Original DATA belongs to this stream, independently of restored
        // ordinary suffix coverage. Fresh recovery receivers explicitly discard
        // their unrelated construction seed before installing foreign history.
    }

    pub(crate) fn compact_branch_transitions_through(
        &self,
        covered: RelationalBranchTransitionSequence,
    ) {
        let mut state = self.branch_transitions.state();
        state.covered = state.covered.max(covered);
    }

    pub(crate) fn retained_branch_transitions(
        &self,
    ) -> (
        RelationalBranchTransitionSequence,
        Vec<Arc<PositionedBranchTransition>>,
    ) {
        let state = self.branch_transitions.state();
        let start = state
            .log
            .partition_point(|entry| entry.positioned.sequence() <= state.covered);
        let end = state
            .log
            .partition_point(|entry| entry.positioned.sequence() <= state.head);
        (
            state.covered,
            state.log[start.min(end)..end]
                .iter()
                .map(|entry| Arc::clone(&entry.positioned))
                .collect(),
        )
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
                covered: state.covered,
                log: state.log.clone(),
                birth: state.birth.clone(),
                installation: state.installation.clone(),
                initial_inputs: state.initial_inputs.clone(),
                home_world_selection: state.home_world_selection.clone(),
                // This existing ordinary detached-copy allocation is not a
                // controlled positive. Capture separately admits its own roster.
                _log_memory: None,
            }),
        })
    }

    fn head(&self) -> RelationalBranchTransitionSequence {
        self.state().head
    }

    fn append(
        &self,
        positioned: PositionedBranchTransition,
        schema: Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
        retained: Option<usize>,
    ) -> Arc<PositionedBranchTransition> {
        let mut state = self.state();
        if let Some(index) = retained {
            // The performed reservation and publication lifecycle guard have
            // excluded any writer since exact original-prefix preparation.
            state.head = positioned.sequence();
            return Arc::clone(&state.log[index].positioned);
        }
        assert!(
            state.log.len() < state.log.capacity(),
            "transition storage was prepared before durability"
        );
        let positioned = Arc::new(positioned);
        state.head = positioned.sequence();
        state.log.push(RetainedBranchTransition {
            positioned: Arc::clone(&positioned),
            schema,
        });
        positioned
    }

    fn state(&self) -> MutexGuard<'_, BranchTransitionStreamState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
