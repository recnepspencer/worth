use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
mod checkpoint_selection;
mod initial_inputs;
pub(crate) use checkpoint_selection::CheckpointPublication;
pub(crate) use initial_inputs::RelationalCommittedInitialInputs;
pub use initial_inputs::RelationalInitialInputsPublicationStop;
use worth_proof::{AuthorityMarker, DurablyCommitted};

use super::{
    DurablePublicationFate, RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedSubject,
    RelationalDurableMedium, RelationalDurablePublicationRefusal,
};
use crate::branch::PositionedBranchTransition;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::history::data::PositionedCanonicalCommit;

/// A runtime's attachment to its durable medium.
///
/// It owns the medium itself. The medium was moved into exactly one runtime
/// and leaves it only by moving into the runtime recovery rebuilds (D1); from
/// then on this attachment, and every port of the runtime that held it,
/// refuses to publish. It also owns the one fact the runtime keeps about the
/// medium between publications: whether an earlier publication's fate is
/// unknown. There is one such slot for commits and branch transitions alike:
/// once either is unresolved, every later publication of either kind refuses
/// with the same answer, so a later commit never overtakes an unresolved
/// delete and a later transition never overtakes an unresolved commit.
#[derive(Debug)]
pub(crate) struct RelationalDurableMediumBinding {
    state: Mutex<BindingState>,
    cold_source: Arc<dyn super::RelationalDurableColdSource>,
    unresolved: Arc<OnceLock<RelationalDurabilityUnresolved>>,
}

#[derive(Debug)]
struct BindingState {
    /// `None` once the medium moved to a recovered runtime.
    medium: Option<Box<dyn RelationalDurableMedium>>,
    unresolved: Arc<OnceLock<RelationalDurabilityUnresolved>>,
}

/// Why the durable step did not make a commit or transition durable.
#[derive(Debug, Clone)]
pub(crate) enum RelationalDurableStepStop {
    /// Nothing durable changed; the position or sequence is not consumed.
    Refused(RelationalDurablePublicationRefusal),
    /// The publication's durable fate is unknown until the runtime is
    /// reopened.
    Unresolved(RelationalDurabilityUnresolved),
}

/// Placement facts issued only after the journal answered Committed.
#[derive(Debug)]
pub(crate) struct RelationalDurableCommitReadmission {
    positioned: Arc<PositionedCanonicalCommit>,
    entry: crate::history::RetainedCanonicalIndex,
    source: Arc<dyn super::RelationalDurableColdSource>,
}
impl RelationalDurableCommitReadmission {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Arc<PositionedCanonicalCommit>,
        crate::history::RetainedCanonicalIndex,
        Arc<dyn super::RelationalDurableColdSource>,
    ) {
        (self.positioned, self.entry, self.source)
    }
}

impl AuthorityMarker for RelationalDurableCommitReadmission {}

/// The exact positioned transition accepted by the medium. Its construction
/// stays in this binding, on the Committed branch only.
#[derive(Debug)]
pub(crate) struct RelationalDurableTransitionPublication {
    positioned: PositionedBranchTransition,
}

impl AuthorityMarker for RelationalDurableTransitionPublication {}

impl RelationalDurableTransitionPublication {
    pub(crate) fn into_positioned(self) -> PositionedBranchTransition {
        self.positioned
    }
}

impl RelationalDurableMediumBinding {
    pub(crate) fn has_unresolved_publication(&self) -> bool {
        self.unresolved.get().is_some()
    }
    pub(crate) fn cold_source(&self) -> Arc<dyn super::RelationalDurableColdSource> {
        self.cold_source.clone()
    }
    pub(super) fn preparation_fate(&self) -> Arc<OnceLock<RelationalDurabilityUnresolved>> {
        self.unresolved.clone()
    }
    pub(crate) fn read_transition(
        &self,
        sequence: crate::branch::RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, super::RelationalDurableTransitionReadFailure>
    {
        self.state()
            .medium
            .as_ref()
            .ok_or(super::RelationalDurableTransitionReadFailure::MediumMoved)?
            .read_transition(sequence)
    }
    pub(crate) fn new(medium: Box<dyn RelationalDurableMedium>) -> Self {
        let unresolved = Arc::new(OnceLock::new());
        let cold_source = medium.cold_source();
        Self {
            cold_source,
            state: Mutex::new(BindingState {
                medium: Some(medium),
                unresolved: unresolved.clone(),
            }),
            unresolved,
        }
    }

    /// Encode `commit` as its canonical artifact and make it durable.
    ///
    /// Called with the patch-position reservation held, after preparation and
    /// before the live cutover, so a refusal leaves no live or durable effect.
    pub(crate) fn publish(
        &self,
        commit: Arc<PositionedCanonicalCommit>,
        roots: &crate::branch::RelationalBranchRootDescriptor,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DurablyCommitted<RelationalDurableCommitReadmission>, RelationalDurableStepStop>
    {
        let mut state = self.state();
        let medium = state.admitting_medium()?;
        let artifact = RelationalCanonicalCommitArtifact::from_positioned(&commit, policy)
            .map_err(|denial| refused(RelationalDurablePublicationRefusal::Encoding(denial)))?;
        let entry = crate::history::RetainedCanonicalIndex::from_publication(
            &commit,
            roots,
            *artifact.payload_digest().as_bytes(),
            execution,
        )
        .map_err(|denial| refused(RelationalDurablePublicationRefusal::Encoding(denial)))?;
        let answer = medium.publish(&artifact);
        state.settle(
            answer,
            RelationalDurabilityUnresolvedSubject::CanonicalCommit {
                position: artifact.position(),
                commit_id: artifact.commit_id(),
                digest: artifact.payload_digest(),
            },
            || RelationalDurableCommitReadmission {
                positioned: commit,
                entry,
                source: self.cold_source.clone(),
            },
        )
    }

    /// Encode `transition` as its artifact and make it durable.
    ///
    /// Called with the patch-position reservation held, after every fallible
    /// preparation step and before any live effect of the transition.
    pub(crate) fn publish_transition(
        &self,
        transition: PositionedBranchTransition,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<DurablyCommitted<RelationalDurableTransitionPublication>, RelationalDurableStepStop>
    {
        let mut state = self.state();
        let medium = state.admitting_medium()?;
        let artifact = RelationalBranchTransitionArtifact::from_positioned(&transition, policy)
            .map_err(|denial| {
                refused(RelationalDurablePublicationRefusal::TransitionEncoding(
                    denial,
                ))
            })?;
        let answer = medium.publish_transition(&artifact);
        state.settle(
            answer,
            RelationalDurabilityUnresolvedSubject::BranchTransition {
                sequence: artifact.sequence(),
                follows: artifact.follows(),
                digest: artifact.payload_digest(),
            },
            || RelationalDurableTransitionPublication {
                positioned: transition,
            },
        )
    }

    /// Move the medium out so recovery can move it into the rebuilt runtime.
    /// Waits for a publication in flight; afterwards this attachment refuses
    /// every publication. An unresolved fate stays behind: the rebuilt
    /// runtime's recovery has decided it. `None` if it already moved.
    pub(crate) fn detach(&self) -> Option<Box<dyn RelationalDurableMedium>> {
        self.state().medium.take()
    }

    /// The in-crate test medium behind this binding, if it is one.
    #[cfg(test)]
    pub(crate) fn in_crate_test_medium(
        &self,
    ) -> Option<std::sync::Arc<crate::tests::support::TestDurableMedium>> {
        self.state()
            .medium
            .as_ref()
            .and_then(|medium| medium.in_crate_test_medium())
    }

    fn state(&self) -> MutexGuard<'_, BindingState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl BindingState {
    /// The medium, when no earlier fate is unresolved and the medium has not
    /// moved.
    fn admitting_medium(&self) -> Result<&dyn RelationalDurableMedium, RelationalDurableStepStop> {
        if let Some(earlier) = self.unresolved.get() {
            return Err(RelationalDurableStepStop::Unresolved(earlier.clone()));
        }
        self.medium
            .as_deref()
            .ok_or_else(|| refused(RelationalDurablePublicationRefusal::MediumMoved))
    }

    /// Record the medium's answer for `subject`; an unknown fate blocks every
    /// later publication.
    fn settle<Publication: AuthorityMarker>(
        &mut self,
        answer: DurablePublicationFate<String>,
        subject: RelationalDurabilityUnresolvedSubject,
        committed: impl FnOnce() -> Publication,
    ) -> Result<DurablyCommitted<Publication>, RelationalDurableStepStop> {
        match answer {
            DurablePublicationFate::Committed => Ok(DurablyCommitted::from_owner(committed())),
            DurablePublicationFate::ProvenNoEffect { detail } => Err(refused(
                RelationalDurablePublicationRefusal::ProvenNoEffect { detail },
            )),
            DurablePublicationFate::Indeterminate { detail } => Err(
                RelationalDurableStepStop::Unresolved(self.record_unresolved(subject, detail)),
            ),
        }
    }

    fn record_unresolved(
        &self,
        subject: RelationalDurabilityUnresolvedSubject,
        detail: String,
    ) -> RelationalDurabilityUnresolved {
        self.unresolved
            .get_or_init(|| RelationalDurabilityUnresolved::new(subject, detail))
            .clone()
    }
}

fn refused(refusal: RelationalDurablePublicationRefusal) -> RelationalDurableStepStop {
    RelationalDurableStepStop::Refused(refusal)
}
