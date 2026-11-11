//! Unwinding restores the same prepaid names to their occurrence owner.
use super::*;
pub(super) struct OpenRecordLoan<'a> {
    destination: &'a mut Option<WorldOpenAttemptRecord>,
    record: Option<WorldRecord>,
}
impl<'a> OpenRecordLoan<'a> {
    pub(super) fn take(
        destination: &'a mut Option<WorldOpenAttemptRecord>,
        owners: super::super::artifact::WorldOccurrenceOwnerEvidence,
        publishing_incarnation: worth_foundational::facade::HomeIncarnation,
    ) -> Self {
        let open = destination
            .take()
            .expect("the prepared occurrence retains its exact names");
        Self {
            destination,
            record: Some(WorldRecord::Occurrence {
                open,
                owners,
                publishing_incarnation,
            }),
        }
    }
    pub(super) fn record(&self) -> &WorldRecord {
        self.record
            .as_ref()
            .expect("the loan still owns the occurrence")
    }
    pub(super) fn commit(mut self) -> WorldRecord {
        self.record
            .take()
            .expect("one completion consumes the occurrence")
    }
}
impl Drop for OpenRecordLoan<'_> {
    fn drop(&mut self) {
        if let Some(record) = self.record.take() {
            match record {
                WorldRecord::Occurrence { open, .. } => *self.destination = Some(open),
                WorldRecord::Owner { .. }
                | WorldRecord::Branch { .. }
                | WorldRecord::OpenAttempt(_)
                | WorldRecord::Released { .. }
                | WorldRecord::Continued { .. }
                | WorldRecord::CreationIntent(_)
                | WorldRecord::BranchCreated(_)
                | WorldRecord::CreationReleased { .. } => {
                    unreachable!("only an actual occurrence can be loaned")
                }
            }
        }
    }
}
