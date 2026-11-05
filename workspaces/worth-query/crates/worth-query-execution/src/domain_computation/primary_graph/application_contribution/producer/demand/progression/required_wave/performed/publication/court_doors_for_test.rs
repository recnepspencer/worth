//! Sealed test doors over this module's private members.
use super::*;

impl PerformedMembers {
    #[doc(hidden)]
    pub fn publication_for_test<'record>(
        &'record mut self,
        readiness: PublicationReadiness,
        receipt: ProducerCommitReceipt,
        mark: impl FnOnce(PublishedMember<'_>),
    ) -> PublicationHandoff<'record> {
        Self::publication(self, readiness, receipt, mark)
    }
}

impl PublishedMember<'_> {
    #[doc(hidden)]
    pub fn performed_for_test(self) {
        Self::performed(self)
    }
}
