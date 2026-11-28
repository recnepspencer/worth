//! Pure unit tests for `CanonicalCursorKey` total order, admitted query
//! contracts, and `HistoricalReadDenial` / `RetainedBasisTokenDecodeError`
//! `Reportable`/`Disposition` contracts.

use worth_execution::report::{
    DenialClass, DiagnosticDetail, Disposition, ReportProjection, Reportable, RetryHint,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_relational::facade::storage::RelationalStorageSubject;

use super::*;

#[test]
fn query_contract_rejects_unadmitted_projection_and_ordering_names() {
    let schema =
        SchemaVersionIdentity::new(SchemaId("s".into()), SchemaVersionId(1), [7; 32]).unwrap();
    let full = HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(1)),
        PROJECTION_CANONICAL_FULL,
        ORDERING_CANONICAL_ASC,
    );
    assert_eq!(
        full.admitted_canonical_projection(),
        Some(AdmittedHistoricalProjection::Full)
    );
    let bad_proj = HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(1)),
        "unrecognized.projection",
        ORDERING_CANONICAL_ASC,
    );
    assert_eq!(bad_proj.admitted_canonical_projection(), None);
    let bad_order = HistoricalQueryContract::new(
        schema,
        CanonicalKeyDomain::Entity,
        Some(KindId(1)),
        PROJECTION_CANONICAL_FULL,
        "priority.desc",
    );
    assert_eq!(bad_order.admitted_canonical_projection(), None);
}

#[test]
fn canonical_cursor_key_orders_strictly_by_logical_coordinates() {
    let start = CanonicalCursorKey::start();
    let e0 = CanonicalCursorKey::after(PartitionId(0), CanonicalKeyDomain::Entity, b"a".to_vec())
        .unwrap();
    let e1 = CanonicalCursorKey::after(PartitionId(0), CanonicalKeyDomain::Entity, b"b".to_vec())
        .unwrap();
    let r0 = CanonicalCursorKey::after(PartitionId(0), CanonicalKeyDomain::Relation, b"a".to_vec())
        .unwrap();
    let p1 = CanonicalCursorKey::after(PartitionId(1), CanonicalKeyDomain::Entity, b"a".to_vec())
        .unwrap();
    assert!(start < e0);
    assert!(e0 < e1);
    assert!(e1 < r0);
    assert!(r0 < p1);
    assert!(CanonicalCursorKey::after(
        PartitionId(0),
        CanonicalKeyDomain::Entity,
        vec![0u8; MAX_CANONICAL_CURSOR_KEY_BYTES + 1]
    )
    .is_none());
}

#[test]
fn historical_read_denial_and_decode_error_map_exact_m4_reportable_dispositions() {
    let schema =
        SchemaVersionIdentity::new(SchemaId("s".into()), SchemaVersionId(2), [1; 32]).unwrap();
    let cases = [
        (
            HistoricalReadDenial::HistoryUnavailable {
                commit_id: CommitId(7),
            },
            Disposition::Final,
            DenialClass::NotFound,
        ),
        (
            HistoricalReadDenial::UnsupportedHistoricalMeaning {
                schema_version: schema,
            },
            Disposition::CorrectInput,
            DenialClass::InvalidInput,
        ),
        (
            HistoricalReadDenial::MalformedToken(RetainedBasisTokenDecodeError::InvalidPrefix),
            Disposition::CorrectInput,
            DenialClass::InvalidInput,
        ),
        (
            HistoricalReadDenial::CorruptStorage {
                subject: RelationalStorageSubject::Region(PartitionId(0)),
            },
            Disposition::Escalate,
            DenialClass::Defect,
        ),
        (
            HistoricalReadDenial::MemoryExhausted {
                requested_bytes: 8192,
                allowance_bytes: 4096,
            },
            Disposition::Retry(RetryHint::AfterBackoff),
            DenialClass::Exhausted,
        ),
        (
            HistoricalReadDenial::RequestStopped,
            Disposition::Retry(RetryHint::Immediate),
            DenialClass::Cancelled,
        ),
    ];
    for (denial, expected, expected_class) in cases {
        assert_eq!(denial.disposition(), expected);
        assert_eq!(denial.denial_class(), expected_class);
        let report = denial.to_operator_report(DiagnosticDetail::Standard);
        assert_eq!(report.disposition(), expected);
        assert_eq!(report.denial_class(), expected_class);
    }
    assert_eq!(
        RetainedBasisTokenDecodeError::InvalidPrefix.disposition(),
        Disposition::CorrectInput
    );
    assert_eq!(
        RetainedBasisTokenDecodeError::IntegrityMismatch.disposition(),
        Disposition::Escalate
    );
    let malformed: HistoricalReadDenial = RetainedBasisTokenDecodeError::MalformedField.into();
    assert_eq!(
        malformed,
        HistoricalReadDenial::MalformedToken(RetainedBasisTokenDecodeError::MalformedField)
    );
    assert_eq!(malformed.disposition(), Disposition::CorrectInput);
    let integrity: HistoricalReadDenial = RetainedBasisTokenDecodeError::IntegrityMismatch.into();
    assert!(matches!(
        integrity,
        HistoricalReadDenial::CorruptStorage { .. }
    ));
    assert_eq!(integrity.disposition(), Disposition::Escalate);
}
