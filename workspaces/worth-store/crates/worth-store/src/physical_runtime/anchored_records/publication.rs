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
use super::{
    AnchoredRecordMember, AnchoredRecordMemberLocator, AnchoredRecordMembership,
    AnchoredRecordName, AnchoredRecordPublicationFailure, AnchoredRecordVersion, AnchoredRecords,
};
use crate::physical_runtime::record_serving::DurableRecordAppendFailure;
use crate::physical_runtime::{AdmittedRecordPlacementPolicy, PhysicalMutationDeadline};

mod admission;
mod batch_capacity;
mod current;
mod member_writes;
pub(super) mod operation;
pub(in crate::physical_runtime) mod step_custody;

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
    /// Drives the same owned phases using this door's existing Signal deadline.
    /// Admission hashes input and may read the selected record; each pending
    /// phase waits only for its current ordinary managed append.
    pub fn publish(
        &self,
        publication: AnchoredRecordPublication,
    ) -> Result<AnchoredRecordPublished, Failure> {
        use operation::{
            AnchoredRecordOperationFailure as Cause, AnchoredRecordPublicationStart as Start,
            AnchoredRecordPublicationStep as Step,
        };
        let mut operation = match self.admit_publication(publication, None) {
            Start::Ready {
                operation,
                recovery: _recovery,
            } => operation,
            Start::AlreadyPublished(receipt) => return Ok(receipt),
            Start::Refused(Cause::Publication(failure)) => return Err(failure),
            Start::Refused(Cause::Busy) => return Err(Failure::PublicationBusy),
            Start::Refused(_) => return Err(Failure::Unresolved),
        };
        loop {
            match operation.drive(self, None) {
                Step::Pending(pending) => {
                    let step = pending.shared.state().accepted.clone();
                    if let Some(step) = step {
                        step.wait();
                    }
                    operation = pending;
                }
                Step::Ready(ready) => operation = ready,
                Step::Completed(receipt) => return Ok(receipt),
                Step::Failed(failure) => return Err(failure.into_publication_failure()),
            }
        }
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
