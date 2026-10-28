use sha2::{Digest, Sha256};
use worth_store_physical_format::{RecordAnchor, MAXIMUM_RECORD_ANCHORS};

use super::frame::JournalRecordHeader;
use super::lookup::read_header;
use super::{
    OrderedJournal, OrderedJournalEntry, OrderedJournalName, OrderedJournalPublication,
    OrderedJournalPublicationFailure, OrderedJournalPublished, OrderedJournalReadFailure,
};
use crate::physical_runtime::record_serving::{AnchoredRecordAppend, DurableRecordAppendFailure};
use crate::physical_runtime::PhysicalRecordId;

type Failure = OrderedJournalPublicationFailure;

impl OrderedJournal<'_> {
    /// Makes one entry durable at this journal's head plus one, or reports
    /// why not.
    ///
    /// Publications are serialized across journals. A retry of an entry that
    /// is already durable writes nothing and answers `AlreadyCommitted`. An
    /// indeterminate publication blocks this journal alone until reopen.
    pub fn publish(
        &self,
        publication: OrderedJournalPublication,
    ) -> Result<OrderedJournalPublished, Failure> {
        let mut state = self.runtime.ordered_journal_state();
        if let Some(position) = state.unresolved.get(&self.name) {
            return Err(Failure::Unresolved {
                position: *position,
            });
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
                Ok(OrderedJournalPublished::AlreadyCommitted(entry))
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
        let anchor_table_full =
            reader.selected_record_anchors().iter().count() == MAXIMUM_RECORD_ANCHORS;
        if head.is_none() && anchor_table_full {
            return Err(Failure::AnchorTableFull);
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
        .ok_or(Failure::Read(OrderedJournalReadFailure::Damaged))?;
        let record = AnchoredRecordAppend {
            record: header.encode_record(&publication.payload),
            anchor: self.name.anchor(),
            expected_generation: head_position,
            idempotency: material(self.name, publication.position, digest),
            placement: publication.placement,
            deadline: publication.deadline,
        }
        .submit(&self.runtime.record_submission())
        .map_err(|failure| {
            if failure.leaves_fate_unresolved() {
                state.unresolved.insert(self.name, publication.position);
            }
            journal_failure(failure)
        })?;
        Ok(OrderedJournalPublished::Committed(OrderedJournalEntry {
            position: publication.position,
            record: PhysicalRecordId::from_persisted(record),
            payload_sha256: digest,
            payload_bytes: publication.payload.len() as u64,
        }))
    }
}

/// The journal's name for each anchored-append failure.
fn journal_failure(failure: DurableRecordAppendFailure) -> Failure {
    match failure {
        DurableRecordAppendFailure::Batch(denial) => Failure::Batch(denial),
        DurableRecordAppendFailure::Idempotency(denial) => Failure::Idempotency(denial),
        DurableRecordAppendFailure::Preparation(outcome) => Failure::Preparation(outcome),
        DurableRecordAppendFailure::ProvenNoEffect(fate) => Failure::ProvenNoEffect(fate),
        DurableRecordAppendFailure::Indeterminate(fate) => Failure::Indeterminate(fate),
        DurableRecordAppendFailure::CompletedWithoutRecords => Failure::CompletedWithoutRecord,
    }
}

/// Idempotency material, domain-separated from the anchored-record door's: a
/// retry of the same entry of the same journal in one incarnation resolves
/// to the original attempt; a different payload at the same position, or the
/// same position of another journal, is a different attempt.
fn material(journal: OrderedJournalName, position: u64, digest: [u8; 32]) -> [u8; 32] {
    let mut sha = Sha256::new();
    sha.update(b"worth.store.ordered-journal.publication.v1");
    sha.update([journal.as_str().len() as u8]);
    sha.update(journal.as_str().as_bytes());
    sha.update(position.to_le_bytes());
    sha.update(digest);
    sha.finalize().into()
}
