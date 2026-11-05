//! Distinguish changed decision facts from a stale exact publication basis.

use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

pub fn assert_product_basis_stale(outcome: WorthQueryApplicationCommitOutcome, cause: &str) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("{cause} must deny before effects: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
}

pub fn assert_changed_decision(outcome: WorthQueryApplicationCommitOutcome, cause: &str) {
    let WorthQueryApplicationCommitOutcome::Stale(stale) = outcome else {
        panic!("{cause} must stale before effects: {outcome:?}");
    };
    assert!(stale.stale_fact_count() > 0);
}
