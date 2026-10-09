use super::*;
use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryResourceLimitRequest,
    WorthQuerySemanticScaleRequest,
};
#[test]
fn managed_lower_admission_refuses_atomic_before_dense_step_conversion() {
    let envelope = WorthQueryExecutionResourceEnvelope::atomic(
        WorthQuerySemanticScaleRequest::selective(),
        WorthQueryResourceLimitRequest::selective(),
        WorthQueryCancellationSafePointFamily::new("attempt").unwrap(),
    );
    let denial = match lower_installed_step_contract(&envelope) {
        Err(denial) => denial,
        Ok(_) => panic!("Atomic is not a managed step"),
    };
    assert_eq!(
        denial.kind,
        WorthQueryManagedLowerAdmissionFailureKind::InstalledStepContract
    );
    assert_eq!(&*denial.detail, "atomic-execution-is-not-bounded-step");
}
