//! The native storage refusal keeps its category and evidence across the Bank facade.
use super::*;
use worth_query_host::facade::primary_graph::{
    RelationalExecutionDenialCause, RelationalStorageUnavailableCause as Cause,
};
#[test]
fn storage_unavailability_keeps_its_bank_preparation_category() {
    // Diagnostic subject data is public serialized meaning, not runtime authority.
    let subject = serde_json::from_str(r#"{"Region":7}"#).unwrap();
    for cause in [
        Cause::MissingMember,
        Cause::SourceUnavailable,
        Cause::Damaged,
        Cause::IntegrityMismatch,
        Cause::LimitExceeded {
            length: 2,
            limit: 1,
        },
        Cause::MemoryExhausted,
    ] {
        let failure = RelationalStorageUnavailable { subject, cause };
        let query = WorthQueryApplicationAttemptDenial::from(
            RelationalExecutionDenialCause::StorageUnavailable(failure),
        );
        let denied = BankCommitPreparationDenial::from(query);
        assert_eq!(
            denied.application_kind(),
            Some(BankApplicationAttemptDenialKind::StorageUnavailable)
        );
        assert_eq!(denied.storage_unavailable(), Some(&failure));
        assert_eq!(
            denied,
            BankCommitPreparationDenial::StorageUnavailable(failure)
        );
    }
}
