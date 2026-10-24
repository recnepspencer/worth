use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::RecordAnchor;

use super::frame::JournalRecordHeader;
use super::lookup::read_header;
use super::{
    CanonicalCommitEntry, CanonicalCommitJournal, CanonicalCommitPublication,
    CanonicalCommitPublicationFailure, CanonicalCommitPublished,
};
use crate::physical_runtime::record_serving::RecordBatchAnchorAdvance;
use crate::physical_runtime::{
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordId,
    RecordAppendBatch,
};

type Failure = CanonicalCommitPublicationFailure;

impl CanonicalCommitJournal<'_> {
    /// Makes one entry durable at the head plus one, or reports why not.
    ///
    /// Publications are serialized per journal. A retry of an entry that is
    /// already durable writes nothing and answers `AlreadyCommitted`.
    pub fn publish(
        &self,
        publication: CanonicalCommitPublication,
    ) -> Result<CanonicalCommitPublished, Failure> {
        let mut state = self.runtime.canonical_journal_state();
        if let Some(position) = state.unresolved {
            return Err(Failure::Unresolved { position });
        }
        if publication.payload.is_empty() {
            return Err(Failure::EmptyPayload);
        }
        let digest: [u8; 32] = Sha256::digest(&publication.payload).into();
        let reader = self.reader().map_err(Failure::Read)?;
        let head = self.head_anchor(&reader);
        let head_position = head.map_or(0, RecordAnchor::generation);
        if publication.position >= 1 && publication.position <= head_position {
            let entry = self
                .locate(&reader, publication.position)
                .map_err(Failure::Read)?;
            return if entry.payload_sha256() == digest {
                Ok(CanonicalCommitPublished::AlreadyCommitted(entry))
            } else {
                Err(Failure::PositionConflict(entry))
            };
        }
        if Some(publication.position) != head_position.checked_add(1) {
            return Err(Failure::PositionGap {
                head: head_position,
                requested: publication.position,
            });
        }
        let head_header = match head {
            Some(anchor) => Some((
                read_header(&reader, self, anchor.record()).map_err(Failure::Read)?,
                anchor.record(),
            )),
            None => None,
        };
        drop(reader);
        let header = JournalRecordHeader::successor(
            self.name,
            head_header
                .as_ref()
                .map(|(header, record)| (header, *record)),
            &publication.payload,
        )
        .filter(|header| header.position == publication.position)
        .ok_or(Failure::Read(super::CanonicalCommitReadFailure::Damaged))?;
        let batch = RecordAppendBatch::builder()
            .push_owned(header.encode_record(&publication.payload))
            .build()
            .and_then(|batch| {
                batch.with_anchor_advance(RecordBatchAnchorAdvance {
                    name: self.name,
                    expected_generation: head_position,
                    record_index: 0,
                })
            })
            .map_err(Failure::Batch)?;
        let submission = self.runtime.record_submission();
        let key = submission
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material(
                self.name.as_str(),
                publication.position,
                digest,
            )))
            .map_err(Failure::Idempotency)?;
        let request = PhysicalMutationRequest::platform_durable(key, publication.deadline);
        let completed = match submission
            .prepare_durable_append(batch, publication.placement, request)
            .into_raw()
        {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                match prepared.execute() {
                    PhysicalMutationOutcome::Completed(completed) => completed,
                    PhysicalMutationOutcome::ProvenNoEffect(fate) => {
                        return Err(Failure::ProvenNoEffect(fate))
                    }
                    PhysicalMutationOutcome::Indeterminate(fate) => {
                        state.unresolved = Some(publication.position);
                        return Err(Failure::Indeterminate(fate));
                    }
                }
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(done)) => done,
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(
                fate,
            )) => return Err(Failure::ProvenNoEffect(fate)),
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
                state.unresolved = Some(publication.position);
                return Err(Failure::Indeterminate(fate));
            }
            other => return Err(Failure::Preparation(other.into())),
        };
        let [record] = completed.persisted_records() else {
            state.unresolved = Some(publication.position);
            return Err(Failure::Read(super::CanonicalCommitReadFailure::Damaged));
        };
        Ok(CanonicalCommitPublished::Committed(CanonicalCommitEntry {
            position: publication.position,
            record: PhysicalRecordId::from_persisted(*record),
            payload_sha256: digest,
            payload_bytes: publication.payload.len() as u64,
        }))
    }
}

/// Idempotency material: a retry of the same entry in one incarnation resolves
/// to the original attempt; a different payload at the same position conflicts.
fn material(journal: &str, position: u64, digest: [u8; 32]) -> [u8; 32] {
    let mut sha = Sha256::new();
    sha.update(b"worth.store.canonical-journal.publication.v1");
    sha.update(journal.as_bytes());
    sha.update(position.to_le_bytes());
    sha.update(digest);
    sha.finalize().into()
}
