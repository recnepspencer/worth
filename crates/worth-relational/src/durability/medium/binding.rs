use std::sync::{Mutex, MutexGuard};

use super::{
    RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedSubject, RelationalDurableMedium,
    RelationalDurablePublication, RelationalDurablePublicationRefusal,
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
}

#[derive(Debug)]
struct BindingState {
    /// `None` once the medium moved to a recovered runtime.
    medium: Option<Box<dyn RelationalDurableMedium>>,
    unresolved: Option<RelationalDurabilityUnresolved>,
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

impl RelationalDurableMediumBinding {
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
        Self {
            state: Mutex::new(BindingState {
                medium: Some(medium),
                unresolved: None,
            }),
        }
    }

    /// Encode `commit` as its canonical artifact and make it durable.
    ///
    /// Called with the patch-position reservation held, after preparation and
    /// before the live cutover, so a refusal leaves no live or durable effect.
    pub(crate) fn publish(
        &self,
        commit: &PositionedCanonicalCommit,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), RelationalDurableStepStop> {
        let mut state = self.state();
        let medium = state.admitting_medium()?;
        let artifact = RelationalCanonicalCommitArtifact::from_positioned(commit, policy)
            .map_err(|denial| refused(RelationalDurablePublicationRefusal::Encoding(denial)))?;
        let answer = medium.publish(&artifact);
        state.settle(
            answer,
            RelationalDurabilityUnresolvedSubject::CanonicalCommit {
                position: artifact.position(),
                commit_id: artifact.commit_id(),
                digest: artifact.payload_digest(),
            },
        )
    }

    /// Encode `transition` as its artifact and make it durable.
    ///
    /// Called with the patch-position reservation held, after every fallible
    /// preparation step and before any live effect of the transition.
    pub(crate) fn publish_transition(
        &self,
        transition: &PositionedBranchTransition,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), RelationalDurableStepStop> {
        let mut state = self.state();
        let medium = state.admitting_medium()?;
        let artifact = RelationalBranchTransitionArtifact::from_positioned(transition, policy)
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
        if let Some(earlier) = self.unresolved.as_ref() {
            return Err(RelationalDurableStepStop::Unresolved(earlier.clone()));
        }
        self.medium
            .as_deref()
            .ok_or_else(|| refused(RelationalDurablePublicationRefusal::MediumMoved))
    }

    /// Record the medium's answer for `subject`; an unknown fate blocks every
    /// later publication.
    fn settle(
        &mut self,
        answer: RelationalDurablePublication,
        subject: RelationalDurabilityUnresolvedSubject,
    ) -> Result<(), RelationalDurableStepStop> {
        match answer {
            RelationalDurablePublication::Committed => Ok(()),
            RelationalDurablePublication::ProvenNoEffect { detail } => Err(refused(
                RelationalDurablePublicationRefusal::ProvenNoEffect { detail },
            )),
            RelationalDurablePublication::Indeterminate { detail } => {
                let fate = RelationalDurabilityUnresolved::new(subject, detail);
                self.unresolved = Some(fate.clone());
                Err(RelationalDurableStepStop::Unresolved(fate))
            }
        }
    }
}

fn refused(refusal: RelationalDurablePublicationRefusal) -> RelationalDurableStepStop {
    RelationalDurableStepStop::Refused(refusal)
}
