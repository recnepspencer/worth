//! An unquantified work refusal stays work at both terminal scratch admissions.
use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[test]
fn subject_scratch_reports_unquantified_exhaustion_as_work() {
    let stop = CompanionPreflightStop::WorkExhaustedWithoutRequirement { maximum: 1 };
    assert_eq!(
        subject_scratch_denial(stop).kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
}

#[test]
fn member_scratch_reports_unquantified_exhaustion_as_work() {
    let stop = CompanionPreflightStop::WorkExhaustedWithoutRequirement { maximum: 1 };
    assert_eq!(
        member_scratch_denial(stop).kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
}
