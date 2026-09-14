use super::*;

#[test]
fn capacity_denial_identifies_the_first_insufficient_dimension() {
    let denial = admit_execution_resource_plan(
        "binding",
        &contract(8),
        &request(8),
        support(7),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::Backpressured
    );
    assert!(denial.detail().contains("ModelSize=7"));
    assert!(denial.detail().contains("required 8"));
}

#[test]
fn capacity_denial_identifies_an_insufficient_resource_dimension() {
    let resource_limited = WorthQueryExecutionResourceEnvelope::new(
        WorthQuerySemanticScaleRequest::bounded(8),
        WorthQueryResourceLimitRequest::bounded(7),
        WorthQueryExecutionMode::Synchronous,
        None,
        safe_point(),
    );
    let denial = admit_execution_resource_plan(
        "binding",
        &contract(8),
        &request(8),
        support_with_capacity(
            resource_limited,
            std::sync::Arc::new(
                WorthQueryFixedExecutionCapacity::mint("resource-limited", 8).unwrap(),
            ),
        ),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::Backpressured
    );
    assert!(denial.detail().contains("ScratchBytes=7"));
    assert!(denial.detail().contains("required 8"));
}
