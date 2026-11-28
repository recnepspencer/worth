use std::error::Error;

use worth_execution::report::{
    DenialClass, DiagnosticDetail, Disposition, ReportDetail, ReportProjection, RetryHint,
};
use worth_execution::{LeaseDenial, MemoryLimitDenial, MemoryLimitLevel, WorkCeilingDenial};

use super::{RuntimeWorldOwnerUnavailable, RuntimeWorldServiceDenial};
use crate::branch::ProductBranchInventoryDenial;
use crate::history::CompositeHistoryCatalogDenial;
use crate::recovery::RuntimeWorldRecoveryDenial;

#[test]
fn report_equality_and_json_exclude_per_denial_context() {
    let project = |used| {
        RuntimeWorldServiceDenial::Denied(
            CompositeHistoryCatalogDenial::MetadataCapacityExhausted {
                maximum: 12,
                used,
                requested: 5,
            },
        )
        .to_operator_report(DiagnosticDetail::Forensic)
    };
    let first = project(8);
    let second = project(9);
    assert_eq!(first, second);
    assert_eq!(first.to_canonical_json(), second.to_canonical_json());
    assert_eq!(
        first.report_code().as_str(),
        "world.history.metadata_capacity_exhausted"
    );
    assert_eq!(first.denial_class(), DenialClass::Exhausted);
    assert_eq!(first.disposition(), Disposition::Final);
    assert_eq!(first.chain().len(), 1);
    assert_eq!(first.chain()[0].detail(), &ReportDetail::Unavailable);
}

#[test]
fn projection_preserves_the_entire_typed_cause_chain() {
    let cause = MemoryLimitDenial::Capacity {
        requested: 5,
        admitted: 4,
        level: MemoryLimitLevel::Process,
    };
    let denial = RuntimeWorldServiceDenial::Denied(ProductBranchInventoryDenial::Request(
        WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(cause)),
    ));
    let mut source = denial.source().unwrap();
    assert!(source.is::<ProductBranchInventoryDenial>());
    source = source.source().unwrap();
    assert!(source.is::<WorkCeilingDenial>());
    source = source.source().unwrap();
    assert!(source.is::<LeaseDenial>());
    source = source.source().unwrap();
    assert_eq!(source.downcast_ref::<MemoryLimitDenial>(), Some(&cause));
    assert!(source.source().is_none());
    let report = denial.to_operator_report(DiagnosticDetail::Redacted);
    assert_eq!(report.chain().len(), 4);
    assert!(report
        .chain()
        .iter()
        .all(|frame| frame.detail() == &ReportDetail::Withheld));
}

#[test]
fn missing_owner_remains_a_source_across_recovery() {
    let unavailable = RuntimeWorldOwnerUnavailable::new();
    let denial = RuntimeWorldServiceDenial::Denied(RuntimeWorldRecoveryDenial::from(unavailable));
    assert_eq!(
        denial
            .source()
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<RuntimeWorldOwnerUnavailable>(),
        Some(&unavailable),
    );
    let report = denial.to_operator_report(DiagnosticDetail::Standard);
    assert_eq!(report.chain().len(), 2);
    assert_eq!(
        report.disposition(),
        Disposition::Retry(RetryHint::WithFreshAuthority)
    );
}
