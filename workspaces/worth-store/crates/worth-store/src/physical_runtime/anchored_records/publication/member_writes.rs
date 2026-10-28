//! New members: planned into record batches before any effect, then written
//! as ordinary unanchored records, one durable root generation per batch.
//!
//! A batch never carries the anchored record: a value's table names its
//! members by persisted record identity, which a batch learns only once it
//! completes. Batches fill to the door's batch capacity in the caller's
//! order; equal keys are written once. Each batch has its own idempotency
//! material, derived from the value's content digest, the batch's position
//! and the runtime incarnation, so a retry of the same value in one
//! incarnation resolves to the batches already written.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::super::{
    AnchoredRecordMember, AnchoredRecordMemberLocator, AnchoredRecordMemberWriteFailure,
    AnchoredRecordName, AnchoredRecordPublicationFailure,
};
use super::admission::AdmittedMembers;
use super::batch_capacity::BatchCapacity;
use crate::physical_runtime::record_serving::{DurableRecordAppend, DurableRecordAppendFailure};
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, PhysicalRecordId,
    PhysicalRecordSubmission, RecordAppendBatch,
};

type Failure = AnchoredRecordPublicationFailure;

/// Where each requested member's locator comes from, in the caller's order.
pub(super) struct MemberResolution {
    sources: Vec<MemberSource>,
}

enum MemberSource {
    Carried(AnchoredRecordMemberLocator),
    /// The index of the unique new content, in write order.
    Written(usize),
}

impl MemberResolution {
    /// Every member's locator in the caller's order, given the locators of
    /// the written contents in write order.
    pub(super) fn resolve(
        &self,
        written: &[AnchoredRecordMemberLocator],
    ) -> Vec<AnchoredRecordMemberLocator> {
        self.sources
            .iter()
            .map(|source| match source {
                MemberSource::Carried(locator) => *locator,
                MemberSource::Written(index) => written[*index],
            })
            .collect()
    }
}

/// Every member batch of one publication, built and ready to submit.
pub(super) struct MemberWritePlan {
    batches: Vec<PlannedMemberBatch>,
    name: AnchoredRecordName,
    generation: u64,
    content: [u8; 32],
    incarnation: u64,
}

struct PlannedMemberBatch {
    batch: RecordAppendBatch,
    /// Key and length of each record, in batch order.
    contents: Vec<([u8; 32], u64)>,
}

/// Unique new contents, grouped into batches within the batch capacity.
#[derive(Default)]
struct BatchGrouping {
    closed: Vec<Vec<([u8; 32], Vec<u8>)>>,
    open: Vec<([u8; 32], Vec<u8>)>,
    open_bytes: u64,
    open_extents: usize,
}

impl MemberWritePlan {
    /// Plans and builds every member batch. Refuses only before any effect.
    ///
    /// Cost: O(members + new member bytes moved, not copied).
    pub(super) fn plan(
        members: AdmittedMembers,
        name: AnchoredRecordName,
        generation: u64,
        content: [u8; 32],
        incarnation: u64,
        capacity: BatchCapacity,
    ) -> Result<(Self, MemberResolution), Failure> {
        let mut written = BTreeMap::new();
        let mut grouping = BatchGrouping::default();
        let mut sources = Vec::with_capacity(members.len());
        for member in members.into_members() {
            let (key, bytes) = match member {
                AnchoredRecordMember::Carried { locator } => {
                    sources.push(MemberSource::Carried(locator));
                    continue;
                }
                AnchoredRecordMember::New { key, bytes } => (key, bytes),
            };
            let next = written.len();
            let index = *written.entry(key).or_insert(next);
            sources.push(MemberSource::Written(index));
            if index == next {
                grouping.push(key, bytes, capacity);
            }
        }
        let batches = grouping
            .into_batches()
            .into_iter()
            .map(PlannedMemberBatch::build)
            .collect::<Result<_, _>>()?;
        Ok((
            Self {
                batches,
                name,
                generation,
                content,
                incarnation,
            },
            MemberResolution { sources },
        ))
    }

    /// Writes every batch in order and returns the written contents'
    /// locators in write order. A failure leaves the anchor where it was.
    pub(super) fn write(
        self,
        submission: &PhysicalRecordSubmission,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Result<Vec<AnchoredRecordMemberLocator>, Failure> {
        let mut locators = Vec::new();
        for (written_batches, planned) in self.batches.into_iter().enumerate() {
            let records = DurableRecordAppend {
                batch: planned.batch,
                idempotency: material(
                    self.name,
                    self.generation,
                    self.content,
                    self.incarnation,
                    written_batches,
                ),
                placement,
                deadline,
            }
            .submit(submission)
            .map_err(|failure| Failure::MemberWrite {
                written_batches,
                failure: member_failure(failure),
            })?;
            locators.extend(records.into_iter().zip(planned.contents).map(
                |(record, (sha256, length))| {
                    AnchoredRecordMemberLocator::new(
                        PhysicalRecordId::from_persisted(record),
                        sha256,
                        length,
                    )
                },
            ));
        }
        Ok(locators)
    }
}

impl BatchGrouping {
    /// Adds one content, closing the open batch first when it is full.
    fn push(&mut self, key: [u8; 32], bytes: Vec<u8>, capacity: BatchCapacity) {
        let length = bytes.len() as u64;
        let extent = usize::from(capacity.is_extent(length));
        let full = self.open.len() == capacity.records
            || self.open_bytes + length > capacity.bytes
            || self.open_extents + extent > capacity.extents;
        if !self.open.is_empty() && full {
            self.closed.push(std::mem::take(&mut self.open));
            self.open_bytes = 0;
            self.open_extents = 0;
        }
        self.open.push((key, bytes));
        self.open_bytes += length;
        self.open_extents += extent;
    }

    fn into_batches(mut self) -> Vec<Vec<([u8; 32], Vec<u8>)>> {
        if !self.open.is_empty() {
            self.closed.push(self.open);
        }
        self.closed
    }
}

impl PlannedMemberBatch {
    fn build(contents: Vec<([u8; 32], Vec<u8>)>) -> Result<Self, Failure> {
        let mut builder = RecordAppendBatch::builder();
        let mut keys = Vec::with_capacity(contents.len());
        for (key, bytes) in contents {
            keys.push((key, bytes.len() as u64));
            builder = builder.push_owned(bytes);
        }
        Ok(Self {
            batch: builder.build().map_err(Failure::Batch)?,
            contents: keys,
        })
    }
}

/// This door's name for a member batch failure.
fn member_failure(failure: DurableRecordAppendFailure) -> AnchoredRecordMemberWriteFailure {
    type Member = AnchoredRecordMemberWriteFailure;
    match failure {
        DurableRecordAppendFailure::Batch(denial) => Member::Batch(denial),
        DurableRecordAppendFailure::Idempotency(denial) => Member::Idempotency(denial),
        DurableRecordAppendFailure::Preparation(outcome) => Member::Preparation(outcome),
        DurableRecordAppendFailure::ProvenNoEffect(fate) => Member::ProvenNoEffect(fate),
        DurableRecordAppendFailure::Indeterminate(fate) => Member::Indeterminate(fate),
        DurableRecordAppendFailure::CompletedWithoutRecords => Member::CompletedWithoutRecords,
    }
}

/// Idempotency material of one member batch, domain-separated from the
/// anchored record's: equal for the same value's same batch in one runtime
/// incarnation. A reopened runtime writes its members afresh: an earlier
/// incarnation's batch is referenced by no anchor, and its binding's fate may
/// still read as unresolved after recovery.
fn material(
    name: AnchoredRecordName,
    generation: u64,
    content: [u8; 32],
    incarnation: u64,
    batch: usize,
) -> [u8; 32] {
    let mut sha = Sha256::new();
    sha.update(b"worth.store.anchored-record.members.v1");
    sha.update(incarnation.to_le_bytes());
    sha.update([name.as_str().len() as u8]);
    sha.update(name.as_str().as_bytes());
    sha.update(generation.to_le_bytes());
    sha.update(content);
    sha.update((batch as u64).to_le_bytes());
    sha.finalize().into()
}
