use crate::history::data::{CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit};
use crate::publication::patch::data::position_denial::RelationalCanonicalPositionAdmissionDenial;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::state::subsystems::history::canonical_publication_routes::position_admission::readmit_positioned_canonical_commit;

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
        readmit_positioned_canonical_commit(position, std::sync::Arc::new(canonical))
            .map(Self::exact)
    }

    /// Readmit the position and placement carried by one original retained cell.
    pub(crate) fn readmit_retained(
        cell: &crate::history::commit::RelationalCommitEnvelopeResidency,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::durability::data::RelationalRecoveryStop> {
        let mut readmitted = Self::exact(cell.demand_positioned(execution, 1 << 30)?);
        if let Some((entry, source)) = cell.journal_placement() {
            readmitted = readmitted
                .with_verified_journal_payload(entry.payload_digest)
                .bind_journal_source(source)?;
        }
        Ok(readmitted)
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
    /// Whether this readmitted commit's symbol dictionary carries `text`.
    pub fn contains_symbol_text(&self, text: &str) -> bool {
        self.positioned
            .envelope()
            .symbol_texts
            .entries
            .iter()
            .any(|(_, entry)| entry.as_str() == text)
    }
    pub(crate) fn positioned(&self) -> &PositionedCanonicalCommit {
        &self.positioned
    }
    pub(crate) fn into_positioned(self) -> PositionedCanonicalCommit {
        self.positioned
    }

    #[cfg(test)]
    pub(crate) fn readmitted_envelope_mut_for_test(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> &mut crate::history::data::CanonicalCommitEnvelopeBody {
        self.journal = None;
        self.positioned.positioned_envelope_mut_for_test(request)
    }
}
