use std::sync::{Mutex, MutexGuard};

use super::{
    RelationalDurabilityUnresolved, RelationalDurableMedium, RelationalDurablePublication,
    RelationalDurablePublicationRefusal,
};
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::history::data::PositionedCanonicalCommit;

/// A runtime's attachment to its durable medium.
///
/// It owns the medium itself. The medium was moved into exactly one runtime
/// and leaves it only by moving into the runtime recovery rebuilds (D1); from
/// then on this attachment, and every port of the runtime that held it,
/// refuses to publish. It also owns the one fact the runtime keeps about the
/// medium between publications: whether an earlier publication's fate is
/// unknown. Once it is, every later publication refuses with the same answer,
/// because a later position cannot be published over an unresolved one.
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

/// Why the durable step did not make a commit durable.
#[derive(Debug, Clone)]
pub(crate) enum RelationalDurableStepStop {
    /// Nothing durable changed; the position is not consumed.
    Refused(RelationalDurablePublicationRefusal),
    /// The commit's durable fate is unknown until the runtime is reopened.
    Unresolved(RelationalDurabilityUnresolved),
}

impl RelationalDurableMediumBinding {
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
        if let Some(earlier) = state.unresolved.as_ref() {
            return Err(RelationalDurableStepStop::Unresolved(earlier.clone()));
        }
        let Some(medium) = state.medium.as_ref() else {
            return Err(RelationalDurableStepStop::Refused(
                RelationalDurablePublicationRefusal::MediumMoved,
            ));
        };
        let artifact = RelationalCanonicalCommitArtifact::from_positioned(commit, policy).map_err(
            |denial| {
                RelationalDurableStepStop::Refused(RelationalDurablePublicationRefusal::Encoding(
                    denial,
                ))
            },
        )?;
        match medium.publish(&artifact) {
            RelationalDurablePublication::Committed => Ok(()),
            RelationalDurablePublication::ProvenNoEffect { detail } => {
                Err(RelationalDurableStepStop::Refused(
                    RelationalDurablePublicationRefusal::ProvenNoEffect { detail },
                ))
            }
            RelationalDurablePublication::Indeterminate { detail } => {
                let fate = RelationalDurabilityUnresolved::new(
                    artifact.position(),
                    artifact.commit_id(),
                    artifact.payload_digest(),
                    detail,
                );
                state.unresolved = Some(fate.clone());
                Err(RelationalDurableStepStop::Unresolved(fate))
            }
        }
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
