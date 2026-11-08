//! A Native-bearing view retains the same immutable record and actual owner.
use super::{NativePublicationMeaning, ReadmittedPublicationRecord};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct NativeReadmittedPublicationRecord {
    pub(super) record: Arc<ReadmittedPublicationRecord>,
    pub(super) native: Arc<NativePublicationMeaning>,
}
impl NativeReadmittedPublicationRecord {
    pub(crate) fn retained(&self) -> Self {
        Self {
            record: self.record.clone(),
            native: self.native.clone(),
        }
    }
    pub(crate) fn record(&self) -> &Arc<ReadmittedPublicationRecord> {
        &self.record
    }
    pub(crate) fn semantic(
        &self,
    ) -> &worth_relational::facade::transactions::RelationalCommitSemanticResult {
        self.native.effect().semantic_result()
    }
}
