//! Original fate comes from the protected World image, not a Native effect.
use super::{WorldCheckpointImage, WorldProgramActivationHistory};
use crate::durability::{AdmittedWorldOpenAttempt, RecoveredCustodyDisposition};
use worth_foundational::facade::{
    CanonicalDigestId, CompositeAttemptAssociation, DurableCompositeCommitIdentity,
    DurableCompositePublicationAttemptIdentity, WorldOriginalFate,
};

/// Borrowed descriptive facts about an actual acknowledged original open.
/// This lends no original receipt, executable meaning, or live authority.
#[derive(Debug)]
pub struct WorldOriginalStateObservation<'a> {
    image: &'a WorldCheckpointImage,
    open: &'a AdmittedWorldOpenAttempt,
}
impl WorldOriginalStateObservation<'_> {
    pub fn association(&self) -> &CompositeAttemptAssociation {
        &self.open.association
    }
    pub fn attempt(&self) -> DurableCompositePublicationAttemptIdentity {
        self.open.association.attempt()
    }
    pub fn reserved_commit(&self) -> DurableCompositeCommitIdentity {
        self.open.commit
    }
    pub fn predecessor(&self) -> DurableCompositeCommitIdentity {
        self.open.expected.commit
    }
    pub fn program_revision(&self) -> CanonicalDigestId {
        CanonicalDigestId::new(self.open.program_revision)
    }
    pub fn fate(&self) -> WorldOriginalFate {
        self.image
            .original_fate(self.attempt())
            .expect("borrowed actual image open")
    }
}
impl WorldProgramActivationHistory {
    /// One immutable source cut; a foreign or absent original has no observation.
    pub fn original_state(
        &self,
        attempt: DurableCompositePublicationAttemptIdentity,
    ) -> Option<WorldOriginalStateObservation<'_>> {
        self.image
            .opens
            .get(&attempt)
            .map(|open| WorldOriginalStateObservation {
                image: &self.image,
                open,
            })
    }
    pub fn original_states(&self) -> impl Iterator<Item = WorldOriginalStateObservation<'_>> {
        self.image
            .opens
            .values()
            .map(|open| WorldOriginalStateObservation {
                image: &self.image,
                open,
            })
    }
}
impl WorldCheckpointImage {
    /// Shared classification for image consumers, including Native projection.
    /// History receives this exact open only from an admitted World occurrence.
    /// A performed but released Native effect never establishes World publication.
    pub(in crate::durability) fn original_fate(
        &self,
        attempt: DurableCompositePublicationAttemptIdentity,
    ) -> Option<WorldOriginalFate> {
        let open = self.opens.get(&attempt)?;
        if self
            .history
            .get(&open.commit)
            .is_some_and(|head| head.represents_open(open.open()))
        {
            return Some(WorldOriginalFate::Published);
        }
        Some(
            match self
                .dispositions
                .get(&attempt)
                .map(|entry| entry.disposition())
            {
                None => WorldOriginalFate::Open,
                Some(RecoveredCustodyDisposition::Continued) => WorldOriginalFate::Continued,
                Some(RecoveredCustodyDisposition::Released(_)) => WorldOriginalFate::Released,
            },
        )
    }
}
