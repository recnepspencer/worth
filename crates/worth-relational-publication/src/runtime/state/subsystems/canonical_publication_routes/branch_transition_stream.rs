//! The runtime's branch transition stream: the sequence of performed fork,
//! archive and delete transitions, and the one place a transition is made
//! durable and cut over (Milestone 2 D8).
//!
//! A transition records under the routes' publication read lock, so a
//! checkpoint selection excludes it exactly as it excludes a commit, and
//! inside the patch-position reservation, so the commit head it follows is
//! fixed while it publishes and no commit can land between its durable step
//! and its cutover. Its sequence is its own: it never consumes a position.

use contracts::branch::transition::sequence::RelationalBranchTransitionSequenceOwnerPort as _;
use std::sync::{Arc, Mutex, MutexGuard};
use worth_relational_branch::internal_port::branch::transition::positioned::{
    PositionedBranchTransition, PositionedBranchTransitionOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;

use contracts::branch::transition::sequence::RelationalBranchTransitionSequence;
use worth_relational_branch::internal_port::branch::transition::durable_step::{
    RelationalBranchTransitionDurableStep, RelationalTransitionPublication,
};
use worth_relational_branch::internal_port::branch::transition::record::RelationalBranchTransition;

pub(crate) use worth_relational_branch::internal_port::branch::transition::original_inputs::{
    OriginalEmptySchemaInstallation, OriginalEmptySchemaObservation, RetainedBranchTransition,
};
use worth_relational_branch::internal_port::branch::transition::stop::BranchTransitionRecordError;

use super::RelationalCanonicalPublicationRoutes;

#[path = "branch_transition_stream/associated_originals.rs"]
pub mod associated_originals;
#[path = "branch_transition_stream/retained_log.rs"]
pub mod retained_log;
#[path = "branch_transition_stream/schema_inputs.rs"]
pub mod schema_inputs;
#[path = "branch_transition_stream/source_recovery.rs"]
pub mod source_recovery;
pub(crate) use worth_relational_durability::internal_port::durability::source_initial_inputs::producer_roster::{OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial};

#[derive(Debug, Default)]
pub struct BranchTransitionStream {
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
        Option<Arc<worth_relational_durability::internal_port::durability::source_initial_inputs::image::InitialInputsImage>>,
    home_world_selection: Option<
        Arc<
            worth_proof::VerifiedHomeWorldSelection<
                Arc<worth_relational_durability::internal_port::durability::home_world_catalog::RelationalWorldSourceCatalog>,
            >,
        >,
    >,
    _log_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Exact transition stream attached to the installed fork port.
#[derive(Debug, Clone)]
pub struct ForkTransitionRoutes(Arc<RelationalCanonicalPublicationRoutes>);

impl ForkTransitionRoutes {
    pub fn new(owner: Arc<RelationalCanonicalPublicationRoutes>) -> Self {
        Self(owner)
    }

    pub fn observe_original_frontiers(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            contracts::publication::patch::data::patch_position::PatchStreamPosition,
            RelationalBranchTransitionSequence,
        ),
        worth_execution::WorkCeilingDenial,
    > {
        self.0.observe_fork_original_frontiers(execution)
    }

    pub fn begin_branch_transition(&self) -> BranchTransitionPublication<'_> {
        self.0.begin_branch_transition()
    }
}

/// Lifecycle admission acquired before any live branch coordination or state.
pub struct BranchTransitionPublication<'routes> {
    routes: &'routes RelationalCanonicalPublicationRoutes,
    _lifecycle: std::sync::RwLockReadGuard<'routes, ()>,
}

impl RelationalCanonicalPublicationRoutes {
    pub fn original_branch_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Option<Arc<PositionedBranchTransition>> {
        let state = self.branch_transitions.state();
        if sequence > state.head {
            return None;
        }
        state
            .log
            .binary_search_by_key(&sequence, |entry| entry.positioned.sequence())
            .ok()
            .map(|index| Arc::clone(&state.log[index].positioned))
    }

    pub fn try_initial_schema_publication(
        &self,
    ) -> std::sync::TryLockResult<std::sync::RwLockWriteGuard<'_, ()>> {
        self.lifecycle.try_write()
    }
    pub(crate) fn begin_branch_transition(&self) -> BranchTransitionPublication<'_> {
        if let Some(control) = self.lifecycle_order.get() {
            control.transition_contact();
        }
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
    pub fn record_with_cutover<D, R, S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        schema: Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
        prepare: impl FnOnce() -> Result<RelationalBranchTransition, D>,
        cutover: impl FnOnce(
            &PositionedBranchTransition,
            Option<
                worth_proof::DurablyCommitted<
                    worth_relational_branch::internal_port::branch::transition::committed_evidence::RelationalCommittedTransitionEvidence,
                >,
            >,
        ) -> R,
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
            .with_schema_carrier(Some(carrier))
            .with_caller_association(step.caller_association().cloned());
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
        Ok(publication.cutover(
            |positioned| {
                self.routes
                    .branch_transitions
                    .append(positioned, schema, retained)
            },
            |positioned, evidence| cutover(&positioned, evidence),
        ))
    }
}

impl RelationalCanonicalPublicationRoutes {
    /// Empty routes that continue this runtime's branch transition stream,
    /// for the routes recovery rebuilds from its recovered inventory.
    pub fn continuing_branch_transitions(&self) -> Self {
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
    pub fn detached_branch_transitions(&self) -> Self {
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
    pub fn branch_transition_head(&self) -> RelationalBranchTransitionSequence {
        self.branch_transitions.head()
    }

    pub fn restore_branch_transition_coverage(&self, covered: RelationalBranchTransitionSequence) {
        let mut state = self.branch_transitions.state();
        state.head = covered;
        state.covered = covered;
        // Original DATA belongs to this stream, independently of restored
        // ordinary suffix coverage. Fresh recovery receivers explicitly discard
        // their unrelated construction seed before installing foreign history.
    }

    pub fn compact_branch_transitions_through(&self, covered: RelationalBranchTransitionSequence) {
        let mut state = self.branch_transitions.state();
        state.covered = state.covered.max(covered);
    }

    pub fn retained_branch_transitions(
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
        schema: Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
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

impl RelationalCanonicalPublicationRoutes {
    #[doc(hidden)]
    pub fn prepare_original_transition_for_test(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        transition: &PositionedBranchTransition,
    ) -> Result<Option<usize>, OriginalBranchSchemaInputsDenial> {
        self.branch_transitions.prepare_append(request, transition)
    }

    #[doc(hidden)]
    pub fn original_transition_count_for_test(&self) -> usize {
        self.branch_transitions.state().log.len()
    }
}
