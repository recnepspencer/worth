//! Native storage refusals preserve their category, diagnostic subject, and evidence.
use super::*;
use worth_relational::facade::{
    identity::PartitionId,
    storage::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
};

#[test]
fn storage_refusal_keeps_its_category_subject_and_original_cause() {
    for cause in [
        Cause::MemoryExhausted,
        Cause::SourceUnavailable,
        Cause::IntegrityMismatch,
    ] {
        let native = RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(PartitionId(7)),
            cause,
        };
        let denied =
            WorthQueryApplicationAttemptDenial::native_storage_unavailable(native, "account owner");
        assert_eq!(
            denied.kind(),
            WorthQueryApplicationAttemptDenialKind::StorageUnavailable
        );
        assert_eq!(denied.subject(), "account owner");
        assert_eq!(denied.storage_unavailable(), Some(&native));
        assert!(denied.native_query_read_stop().is_none());
        assert!(denied.allocation_denial().is_none());
    }
}
