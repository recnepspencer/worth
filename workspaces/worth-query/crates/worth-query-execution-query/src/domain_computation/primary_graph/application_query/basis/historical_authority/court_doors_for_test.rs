//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationHistoricalRead {
    #[doc(hidden)]
    pub fn at_application_commit_for_test(receipt: &WorthQueryApplicationCommitReceipt) -> Self {
        Self::at_application_commit(receipt)
    }
}

impl<Schema> WorthQueryApplicationHistoricalBasis<Schema> {
    #[doc(hidden)]
    pub fn release_for_test(self) -> WorthQueryApplicationHistoricalBasisReleaseReceipt {
        Self::release(self)
    }
}

impl<Schema> WorthQueryApplicationHistoricalBasis<Schema> {
    #[doc(hidden)]
    pub fn version_id_for_test(&self) -> worth_relational::facade::identity::VersionId {
        Self::version_id(self)
    }
}

impl WorthQueryApplicationHistoricalBasisReleaseReceipt {
    #[doc(hidden)]
    pub fn released_for_test(&self) -> bool {
        Self::released(self)
    }
}
