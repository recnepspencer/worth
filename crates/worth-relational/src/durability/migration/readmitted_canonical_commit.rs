use crate::history::data::{CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalCanonicalPositionAdmissionDenial;

/// A canonical commit that passed current readmission at its stream
/// position. Only Relational readmission doors construct it; holding one
/// confers no runtime authority beyond read access to the admitted commit.
#[derive(Debug, Clone)]
pub struct ReadmittedCanonicalCommit {
    positioned: PositionedCanonicalCommit,
    journal: Option<JournalReadmission>,
}

#[derive(Debug, Clone)]
enum JournalReadmission {
    Verified([u8; 32]),
    Placed {
        digest: [u8; 32],
        source: std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    },
}

impl ReadmittedCanonicalCommit {
    pub(crate) fn exact(positioned: PositionedCanonicalCommit) -> Self {
        Self {
            positioned,
            journal: None,
        }
    }

    pub(in crate::durability) fn with_verified_journal_payload(mut self, digest: [u8; 32]) -> Self {
        self.journal = Some(JournalReadmission::Verified(digest));
        self
    }

    pub(in crate::durability) fn bind_journal_source(
        mut self,
        source: std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<Self, crate::durability::data::DurabilityError> {
        let Some(JournalReadmission::Verified(digest)) = self.journal.take() else {
            return Err(crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCanonicalArtifact,
                "journal placement requires verified canonical frame readmission",
            ));
        };
        self.journal = Some(JournalReadmission::Placed { digest, source });
        Ok(self)
    }

    pub(in crate::durability) fn journal_placement(
        &self,
    ) -> Option<(
        [u8; 32],
        std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    )> {
        match &self.journal {
            Some(JournalReadmission::Placed { digest, source }) => Some((*digest, source.clone())),
            _ => None,
        }
    }

    pub(crate) fn readmit_current(
        position: PatchStreamPosition,
        canonical: CanonicalCommitEnvelope,
    ) -> Result<Self, RelationalCanonicalPositionAdmissionDenial> {
        crate::runtime::readmit_positioned_canonical_commit(
            position,
            std::sync::Arc::new(canonical),
        )
        .map(Self::exact)
    }

    pub fn position(&self) -> PatchStreamPosition {
        self.positioned.position()
    }
    pub fn commit_id(&self) -> CommitId {
        self.positioned.envelope().commit.commit_id
    }
    pub fn envelope(&self) -> &CanonicalCommitEnvelope {
        self.positioned.envelope()
    }
    pub(crate) fn positioned(&self) -> &PositionedCanonicalCommit {
        &self.positioned
    }
    pub(crate) fn into_positioned(self) -> PositionedCanonicalCommit {
        self.positioned
    }

    #[cfg(test)]
    pub(crate) fn envelope_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelope {
        self.journal = None;
        self.positioned.envelope_mut_for_test()
    }
}
