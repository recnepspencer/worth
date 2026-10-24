use std::sync::{Arc, Mutex};

use super::{
    RelationalDurabilityUnresolved, RelationalDurableMedium, RelationalDurablePublication,
    RelationalDurablePublicationRefusal,
};
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::history::data::PositionedCanonicalCommit;

/// A runtime's attachment to its durable medium.
///
/// It owns the one fact the runtime keeps about the medium between
/// publications: whether an earlier publication's fate is unknown. Once it is,
/// every later publication refuses with the same answer, because a later
/// position cannot be published over an unresolved one.
#[derive(Debug)]
pub(crate) struct RelationalDurableMediumBinding {
    medium: Arc<dyn RelationalDurableMedium>,
    unresolved: Mutex<Option<RelationalDurabilityUnresolved>>,
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
    pub(crate) fn new(medium: Arc<dyn RelationalDurableMedium>) -> Self {
        Self {
            medium,
            unresolved: Mutex::new(None),
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
        let mut unresolved = self
            .unresolved
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(earlier) = unresolved.as_ref() {
            return Err(RelationalDurableStepStop::Unresolved(earlier.clone()));
        }
        let artifact = RelationalCanonicalCommitArtifact::from_positioned(commit, policy).map_err(
            |denial| {
                RelationalDurableStepStop::Refused(RelationalDurablePublicationRefusal::Encoding(
                    denial,
                ))
            },
        )?;
        match self.medium.publish(&artifact) {
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
                *unresolved = Some(fate.clone());
                Err(RelationalDurableStepStop::Unresolved(fate))
            }
        }
    }

    /// The medium itself, for a recovered runtime to rebind to.
    pub(crate) fn medium(&self) -> Arc<dyn RelationalDurableMedium> {
        Arc::clone(&self.medium)
    }
}
