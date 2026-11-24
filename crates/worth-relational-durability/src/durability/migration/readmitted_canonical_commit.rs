use branch::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyFieldsPort as _,
};
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use branch::runtime::state::subsystems::history::canonical_publication_routes::position_admission::readmit_positioned_canonical_commit;
use contracts::history::data::coordinates::CommitId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::publication::patch::data::position_denial::RelationalCanonicalPositionAdmissionDenial;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port as storage;

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
        source: std::sync::Arc<
            dyn storage::durability::medium::cold_source::RelationalDurableColdSource,
        >,
    },
}

impl ReadmittedCanonicalCommit {
    pub(crate) fn exact(positioned: PositionedCanonicalCommit) -> Self {
        Self {
            positioned,
            journal: None,
        }
    }

    pub(in crate::internal_port::durability) fn with_verified_journal_payload(
        mut self,
        digest: [u8; 32],
    ) -> Self {
        self.journal = Some(JournalReadmission::Verified(digest));
        self
    }

    pub(in crate::internal_port::durability) fn bind_journal_source(
        mut self,
        source: std::sync::Arc<
            dyn storage::durability::medium::cold_source::RelationalDurableColdSource,
        >,
    ) -> Result<Self, contracts::durability::data::recovery_errors::DurabilityError> {
        let Some(JournalReadmission::Verified(digest)) = self.journal.take() else {
            return Err(contracts::durability::data::recovery_errors::DurabilityError::new(
                contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCanonicalArtifact,
                "journal placement requires verified canonical frame readmission",
            ));
        };
        self.journal = Some(JournalReadmission::Placed { digest, source });
        Ok(self)
    }

    pub(in crate::internal_port::durability) fn journal_placement(
        &self,
    ) -> Option<(
        [u8; 32],
        std::sync::Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>,
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
        cell: &branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, contracts::durability::data::recovery_stop::RelationalRecoveryStop> {
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
            .owner_symbol_texts()
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

    fn readmitted_envelope_mut_for_test(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> &mut branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody {
        self.journal = None;
        crate::durability::canonical_commit_artifact::checkpoint_body::mutable_positioned_body_for_test(&mut self.positioned, request)
    }
}

#[doc(hidden)]
pub trait ReadmittedCanonicalCommitOwnerPort: Sized {
    fn into_positioned_for_test(self) -> PositionedCanonicalCommit;

    fn exact_for_test(
        positioned: worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit,
    ) -> Self;
    fn readmit_current(
        position: PatchStreamPosition,
        canonical: CanonicalCommitEnvelope,
    ) -> Result<Self, RelationalCanonicalPositionAdmissionDenial>;

    fn readmitted_envelope_mut_for_test(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> &mut branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody;

    fn readmit_retained(
        cell: &branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, contracts::durability::data::recovery_stop::RelationalRecoveryStop>;

    fn positioned(&self) -> &PositionedCanonicalCommit;
}

impl ReadmittedCanonicalCommitOwnerPort for ReadmittedCanonicalCommit {
    fn into_positioned_for_test(self) -> PositionedCanonicalCommit {
        self.into_positioned()
    }

    fn exact_for_test(
        positioned: worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit,
    ) -> Self {
        Self::exact(positioned)
    }

    fn readmit_current(
        position: PatchStreamPosition,
        canonical: CanonicalCommitEnvelope,
    ) -> Result<Self, RelationalCanonicalPositionAdmissionDenial> {
        Self::readmit_current(position, canonical)
    }

    fn readmitted_envelope_mut_for_test(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> &mut branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody {
        self.readmitted_envelope_mut_for_test(request)
    }

    fn readmit_retained(
        cell: &branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, contracts::durability::data::recovery_stop::RelationalRecoveryStop> {
        Self::readmit_retained(cell, execution)
    }

    fn positioned(&self) -> &PositionedCanonicalCommit {
        self.positioned()
    }
}
