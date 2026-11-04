//! A single owner-admitted occurrence correlates meaning and its source location.
use super::AdmittedWorldOpenAttempt;
use crate::durability::{WorldOpenAttemptRecord, WorldRelationalEvidence};
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableCompositePublicationAttemptIdentity,
    DurableWorldOwnerIdentity, HomeIncarnation,
};

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldOccurrenceLocation {
    owner: DurableWorldOwnerIdentity,
    attempt: DurableCompositePublicationAttemptIdentity,
    commit: DurableCompositeCommitIdentity,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldOccurrenceLocation {
    pub(crate) fn owner(&self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub(crate) fn attempt(&self) -> DurableCompositePublicationAttemptIdentity {
        self.attempt
    }
    pub(crate) fn commit(&self) -> DurableCompositeCommitIdentity {
        self.commit
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldOccurrence {
    open: AdmittedWorldOpenAttempt,
    relational: WorldRelationalEvidence,
    publishing_incarnation: HomeIncarnation,
    location: AdmittedWorldOccurrenceLocation,
}
impl AdmittedWorldOccurrence {
    /// Only the parent journal admission calls this after validating its exact
    /// source SHA, frame, contiguous position, owner and complete graph edge.
    pub(super) fn from_validated_source(
        open: AdmittedWorldOpenAttempt,
        relational: WorldRelationalEvidence,
        publishing_incarnation: HomeIncarnation,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        let location = AdmittedWorldOccurrenceLocation {
            owner: open.association.attempt().owner(),
            attempt: open.association.attempt(),
            commit: open.commit,
            position,
            digest,
        };
        Self {
            open,
            relational,
            publishing_incarnation,
            location,
        }
    }
    pub(crate) fn admitted_open(&self) -> &AdmittedWorldOpenAttempt {
        &self.open
    }
    pub(crate) fn from_committed(
        open: &AdmittedWorldOpenAttempt,
        committed: &crate::durability::WorldCommittedPublication,
    ) -> Result<Self, super::WorldReadmissionDenial> {
        let crate::durability::WorldRecord::Occurrence {
            open: actual,
            relational,
            publishing_incarnation,
        } = &committed.subject().record
        else {
            return Err(super::WorldReadmissionDenial::OccurrenceEvidenceMismatch);
        };
        if actual != open.open() {
            return Err(super::WorldReadmissionDenial::OccurrenceEvidenceMismatch);
        }
        Ok(Self::from_validated_source(
            open.clone(),
            relational.clone(),
            *publishing_incarnation,
            committed.position(),
            *committed.digest().bytes(),
        ))
    }
    pub(crate) fn open(&self) -> &WorldOpenAttemptRecord {
        self.open.open()
    }
    pub(crate) fn relational(&self) -> &WorldRelationalEvidence {
        &self.relational
    }
    pub(crate) fn publishing_incarnation(&self) -> HomeIncarnation {
        self.publishing_incarnation
    }
    pub(crate) fn location(&self) -> &AdmittedWorldOccurrenceLocation {
        &self.location
    }
}
