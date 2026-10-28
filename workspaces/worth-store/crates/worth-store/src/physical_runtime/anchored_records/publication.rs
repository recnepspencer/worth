//! Publishing one anchored value with its members.
//!
//! Phases, in order: admission of the request (payload, member keys and
//! sizes), reading the current anchor and value (identical retry, generation,
//! anchor table, carried membership), planning every batch, and only then the
//! effects: new member batches, each its own durable root generation, and
//! finally the anchored record with its anchor advance, the single commit
//! point. Every refusal before the first member batch has no effect.

use sha2::{Digest, Sha256};

use super::frame::AnchoredRecordHeader;
use super::membership::{SelectedRecord, ValueMembership};
use super::{
    AnchoredRecordMember, AnchoredRecordMemberLocator, AnchoredRecordMembership,
    AnchoredRecordName, AnchoredRecordPublicationFailure, AnchoredRecordVersion, AnchoredRecords,
};
use crate::physical_runtime::record_serving::{AnchoredRecordAppend, DurableRecordAppendFailure};
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, RecordAppendDenial,
};

mod admission;
mod batch_capacity;
mod current;
mod member_writes;

use admission::AdmittedMembers;
use batch_capacity::BatchCapacity;
use current::{CurrentDecision, RequestedPayload};
use member_writes::MemberWritePlan;

type Failure = AnchoredRecordPublicationFailure;

/// One requested replacement: the value at `expected_generation` (zero when
/// the record is absent) becomes `payload` with `members` at the next
/// generation.
pub struct AnchoredRecordPublication {
    name: AnchoredRecordName,
    expected_generation: u64,
    payload: Vec<u8>,
    members: Vec<AnchoredRecordMember>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
}

impl AnchoredRecordPublication {
    /// A value with no members.
    pub fn new(
        name: AnchoredRecordName,
        expected_generation: u64,
        payload: Vec<u8>,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Self {
        Self {
            name,
            expected_generation,
            payload,
            members: Vec::new(),
            placement,
            deadline,
        }
    }

    /// The value's members, in the order the value lists them.
    pub fn with_members(mut self, members: Vec<AnchoredRecordMember>) -> Self {
        self.members = members;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchoredRecordPublished {
    /// This call made the value durable.
    Published(AnchoredRecordMembership),
    /// The identical value and member contents were already durable at the
    /// requested generation; nothing was written.
    AlreadyPublished(AnchoredRecordMembership),
}

impl AnchoredRecordPublished {
    pub const fn membership(&self) -> &AnchoredRecordMembership {
        match self {
            Self::Published(membership) | Self::AlreadyPublished(membership) => membership,
        }
    }
    pub const fn version(&self) -> AnchoredRecordVersion {
        self.membership().version
    }
    /// Every member's locator, in the caller's order.
    pub fn members(&self) -> &[AnchoredRecordMemberLocator] {
        &self.membership().members
    }
}

impl AnchoredRecords<'_> {
    /// Makes `payload` and its members the value of its name at
    /// `expected_generation + 1` in one durable root decision, or reports why
    /// not.
    ///
    /// Publications are serialized across names. Every refusal before the
    /// first member batch has no effect. A retry of a value whose payload and
    /// member contents are already durable at the requested generation
    /// writes nothing and answers `AlreadyPublished`.
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
        let capacity = BatchCapacity::of(self.runtime, publication.placement);
        let members = AdmittedMembers::admit(publication.members, capacity)?;
        let payload = RequestedPayload {
            sha256: Sha256::digest(&publication.payload).into(),
            bytes: publication.payload.len() as u64,
        };
        let target = match self.decide_against_current(
            name,
            publication.expected_generation,
            payload,
            &members,
        )? {
            CurrentDecision::AlreadyPublished(membership) => {
                return Ok(AnchoredRecordPublished::AlreadyPublished(membership))
            }
            CurrentDecision::Replace { target } => target,
        };
        let header = AnchoredRecordHeader::describe(
            name.anchor(),
            target,
            &publication.payload,
            members.len(),
        );
        if header.record_bytes() > capacity.bytes {
            return Err(Failure::Batch(RecordAppendDenial::BatchByteLimitExceeded));
        }
        let content = members.content_digest(name, target, payload.sha256);
        let incarnation = self.runtime.runtime_identity().get();
        let (plan, resolution) =
            MemberWritePlan::plan(members, name, target, content, incarnation, capacity)?;
        let written = plan.write(
            &self.runtime.record_submission(),
            publication.placement,
            publication.deadline,
        )?;
        let locators = resolution.resolve(&written);
        let record = header.encode_record(&locators, &publication.payload);
        let record_sha256: [u8; 32] = Sha256::digest(&record).into();
        let persisted = AnchoredRecordAppend {
            record,
            anchor: name.anchor(),
            expected_generation: publication.expected_generation,
            idempotency: material(name, target, record_sha256, incarnation),
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
        let selected = SelectedRecord {
            record: persisted,
            record_sha256,
            generation: target,
        };
        let value = self.remember(name, ValueMembership::new(selected, header, locators));
        Ok(AnchoredRecordPublished::Published(value.described(name)))
    }
}

/// This door's name for each anchored-append failure.
fn door_failure(failure: DurableRecordAppendFailure) -> Failure {
    match failure {
        DurableRecordAppendFailure::Batch(denial) => Failure::Batch(denial),
        DurableRecordAppendFailure::Idempotency(denial) => Failure::Idempotency(denial),
        DurableRecordAppendFailure::Preparation(outcome) => Failure::Preparation(outcome),
        DurableRecordAppendFailure::ProvenNoEffect(fate) => Failure::ProvenNoEffect(fate),
        DurableRecordAppendFailure::Indeterminate(fate) => Failure::Indeterminate(fate),
        DurableRecordAppendFailure::CompletedWithoutRecords => Failure::CompletedWithoutRecord,
    }
}

/// Idempotency material of the anchored record, domain-separated from the
/// journal's and the members': a retry of the same record bytes at the same
/// generation in one runtime incarnation resolves to the original attempt;
/// different bytes, or another incarnation, are a new attempt. After a
/// reopen, durable state (the anchor) answers a retry, never an earlier
/// incarnation's binding, whose fate may still read as unresolved.
fn material(
    name: AnchoredRecordName,
    generation: u64,
    record_sha256: [u8; 32],
    incarnation: u64,
) -> [u8; 32] {
    let mut sha = Sha256::new();
    sha.update(b"worth.store.anchored-record.publication.v2");
    sha.update(incarnation.to_le_bytes());
    sha.update([name.as_str().len() as u8]);
    sha.update(name.as_str().as_bytes());
    sha.update(generation.to_le_bytes());
    sha.update(record_sha256);
    sha.finalize().into()
}
