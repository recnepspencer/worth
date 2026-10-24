use sha2::{Digest, Sha256};
use worth_store_physical_format::{RecordAnchor, MAXIMUM_RECORD_ANCHORS};

use super::frame::AnchoredRecordHeader;
use super::read::version;
use super::{
    AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublicationFailure,
    AnchoredRecordPublished, AnchoredRecordVersion, AnchoredRecords,
};
use crate::physical_runtime::record_serving::{AnchoredRecordAppend, AnchoredRecordAppendFailure};
use crate::physical_runtime::PhysicalRecordId;

type Failure = AnchoredRecordPublicationFailure;

impl AnchoredRecords<'_> {
    /// Makes `payload` the value of its name at `expected_generation + 1` in
    /// one durable root decision, or reports why not.
    ///
    /// Publications are serialized across names. Every refusal before the
    /// append has no effect. A retry of a value that is already durable at the
    /// requested generation writes nothing and answers `AlreadyPublished`.
    pub fn publish(
        &self,
        publication: AnchoredRecordPublication,
    ) -> Result<AnchoredRecordPublished, Failure> {
        let mut state = self.runtime.anchored_record_state();
        let name = publication.name;
        if state.unresolved.contains(&name.anchor()) {
            return Err(Failure::Unresolved);
        }
        if publication.payload.is_empty() {
            return Err(Failure::EmptyPayload);
        }
        let reader = self.reader().map_err(Failure::Read)?;
        let anchors = reader.selected_record_anchors();
        drop(reader);
        let current = anchors.get(name.anchor());
        let current_generation = current.map_or(0, RecordAnchor::generation);
        let Some(target) = publication.expected_generation.checked_add(1) else {
            return Err(Failure::GenerationConflict {
                current: current_generation,
            });
        };
        let header = AnchoredRecordHeader::describe(name.anchor(), target, &publication.payload);
        if let Some(anchor) = current.filter(|anchor| {
            anchor.generation() == target
                && anchor.record_payload_sha256() == header.record_sha256(&publication.payload)
        }) {
            return Ok(AnchoredRecordPublished::AlreadyPublished(version(
                name, anchor, &header,
            )));
        }
        if current_generation != publication.expected_generation {
            return Err(Failure::GenerationConflict {
                current: current_generation,
            });
        }
        if current.is_none() && anchors.iter().count() == MAXIMUM_RECORD_ANCHORS {
            return Err(Failure::AnchorTableFull);
        }
        let record = AnchoredRecordAppend {
            record: header.encode_record(&publication.payload),
            anchor: name.anchor(),
            expected_generation: publication.expected_generation,
            idempotency: material(name, target, header.payload_sha256),
            placement: publication.placement,
            deadline: publication.deadline,
        }
        .submit(&self.runtime.record_submission())
        .map_err(|failure| {
            if failure.leaves_fate_unresolved() {
                state.unresolved.insert(name.anchor());
            }
            door_failure(failure)
        })?;
        Ok(AnchoredRecordPublished::Published(AnchoredRecordVersion {
            name,
            generation: target,
            record: PhysicalRecordId::from_persisted(record),
            payload_sha256: header.payload_sha256,
            payload_bytes: header.payload_bytes,
        }))
    }
}

/// This door's name for each anchored-append failure.
fn door_failure(failure: AnchoredRecordAppendFailure) -> Failure {
    match failure {
        AnchoredRecordAppendFailure::Batch(denial) => Failure::Batch(denial),
        AnchoredRecordAppendFailure::Idempotency(denial) => Failure::Idempotency(denial),
        AnchoredRecordAppendFailure::Preparation(outcome) => Failure::Preparation(outcome),
        AnchoredRecordAppendFailure::ProvenNoEffect(fate) => Failure::ProvenNoEffect(fate),
        AnchoredRecordAppendFailure::Indeterminate(fate) => Failure::Indeterminate(fate),
        AnchoredRecordAppendFailure::CompletedWithoutRecord => Failure::CompletedWithoutRecord,
    }
}

/// Idempotency material, domain-separated from the journal's: a retry of the
/// same value at the same generation in one incarnation resolves to the
/// original attempt; a different value at that generation is a new attempt.
fn material(name: AnchoredRecordName, generation: u64, payload_sha256: [u8; 32]) -> [u8; 32] {
    let mut sha = Sha256::new();
    sha.update(b"worth.store.anchored-record.publication.v1");
    sha.update([name.as_str().len() as u8]);
    sha.update(name.as_str().as_bytes());
    sha.update(generation.to_le_bytes());
    sha.update(payload_sha256);
    sha.finalize().into()
}
