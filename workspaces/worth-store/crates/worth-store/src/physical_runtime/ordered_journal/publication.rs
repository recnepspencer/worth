//! One owned journal algorithm; synchronous callers drive the same operation.
use super::frame::JournalRecordHeader;
use super::lookup::read_header;
use super::*;
use crate::physical_runtime::record_serving::{AnchoredRecordAppend, DurableRecordAppendFailure};
use sha2::{Digest, Sha256};
worth_proof::authority_marker!(OrderedJournalPublicationAuthority);
mod admission;
mod operation;
mod outcome;
mod progression;
mod recovery;
mod settlement;
pub(in crate::physical_runtime) use operation::JournalPublicationShared;
pub use operation::{
    OrderedJournalPublicationOperation, OrderedJournalPublicationStart,
    OrderedJournalPublicationStep,
};
pub use outcome::*;
pub use recovery::{
    OrderedJournalPublicationRecovery, OrderedJournalPublicationResume, OrderedJournalStoppedPoll,
    OrderedJournalStoppedPublication,
};
type Failure = OrderedJournalPublicationFailure;
impl OrderedJournal<'_> {
    pub fn publish(
        &self,
        publication: OrderedJournalPublication,
    ) -> Result<OrderedJournalPublished, Failure> {
        let mut operation = match self.start_owned(publication, None) {
            OrderedJournalPublicationStart::Ready {
                operation,
                recovery: _recovery,
            } => operation,
            OrderedJournalPublicationStart::AlreadyCommitted(done) => return Ok(done.published()),
            OrderedJournalPublicationStart::Refused(
                OrderedJournalOperationFailure::Publication(failure),
            ) => return Err(failure),
            OrderedJournalPublicationStart::Refused(cause) => {
                return Err(Failure::OwnedStart(Box::new(cause)))
            }
        };
        loop {
            match operation.step(self) {
                OrderedJournalPublicationStep::Pending(same) => {
                    same.shared.wait();
                    operation = same;
                }
                OrderedJournalPublicationStep::Completed(done) => return Ok(done.published()),
                OrderedJournalPublicationStep::Failed(failure) => return Err(failure.into_legacy()),
            }
        }
    }
}
/// The journal's name for each anchored-append failure.
pub(super) fn journal_failure(failure: DurableRecordAppendFailure) -> Failure {
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
pub(super) fn material(journal: OrderedJournalName, position: u64, digest: [u8; 32]) -> [u8; 32] {
    let mut sha = Sha256::new();
    sha.update(b"worth.store.ordered-journal.publication.v1");
    sha.update([journal.as_str().len() as u8]);
    sha.update(journal.as_str().as_bytes());
    sha.update(position.to_le_bytes());
    sha.update(digest);
    sha.finalize().into()
}
