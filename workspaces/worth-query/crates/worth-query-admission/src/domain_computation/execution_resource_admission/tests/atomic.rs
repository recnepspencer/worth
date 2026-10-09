use super::*;
use std::sync::Arc;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary;
type Envelope = WorthQueryExecutionResourceEnvelope;
type Scale = WorthQuerySemanticScaleRequest;
type Limits = WorthQueryResourceLimitRequest;
type Axis = WorthQuerySemanticScaleAxis;
type Dimension = WorthQueryResourceDimension;
type Kind = WorthQueryExecutionResourceAdmissionDenialKind;
fn atomic(scale: Scale, limits: Limits) -> Envelope {
    Envelope::atomic(scale, limits, safe_point())
}
fn declared(envelope: Envelope) -> WorthQueryExecutionResourceContract {
    WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new("atomic").unwrap(),
        envelope,
        WorthQueryExecutionProviderRequirements::new(provider(), access(), allocator()),
    )])
    .unwrap()
}
fn atomic_request(envelope: &Envelope) -> WorthQueryExecutionResourceRequest {
    WorthQueryExecutionResourceRequest::atomic(
        envelope.scale_ceilings().clone(),
        envelope.resource_ceilings().clone(),
        safe_point(),
    )
    .unwrap()
}
fn admit(
    required: &Envelope,
    supported: Envelope,
) -> Result<WorthQueryAdmittedExecutionResourcePlan, WorthQueryExecutionResourceAdmissionDenial> {
    admit_execution_resource_plan(
        "binding",
        &declared(required.clone()),
        &atomic_request(required),
        support_with_capacity(
            supported,
            Arc::new(WorthQueryFixedExecutionCapacity::new("atomic", 1).unwrap()),
        ),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
}
#[test]
fn atomic_present_demands_require_present_support_without_dense_getters() {
    let required = atomic(
        Scale::selective().with(Axis::CandidateItems, 0),
        Limits::selective().with(Dimension::RetainedBytes, 0),
    );
    assert!(admit(&required, required.clone()).is_ok());
    for missing in [
        atomic(Scale::selective(), required.resource_ceilings().clone()),
        atomic(required.scale_ceilings().clone(), Limits::selective()),
    ] {
        let denial = admit(&required, missing).unwrap_err();
        assert_eq!(denial.kind(), &Kind::Backpressured);
        assert!(denial.detail().contains("does not support required"));
    }
    let no_demand = atomic(Scale::selective(), Limits::selective());
    assert!(admit(&no_demand, required.clone()).is_ok());
    let work = atomic(
        required.scale_ceilings().clone().with(Axis::WorkItems, 9),
        required.resource_ceilings().clone(),
    );
    assert!(
        admit(&work, required.clone()).is_ok(),
        "only WorkItems omission keeps its existing no-policy meaning"
    );
    let denial = admit(
        &work,
        atomic(
            required.scale_ceilings().clone().with(Axis::WorkItems, 0),
            required.resource_ceilings().clone(),
        ),
    )
    .unwrap_err();
    assert_eq!(denial.kind(), &Kind::Backpressured);
    let request = atomic_request(&no_demand).allow_mode(WorthQueryExecutionMode::Asynchronous);
    let denial = admit_execution_resource_plan(
        "binding",
        &declared(no_demand.clone()),
        &request,
        support_with_capacity(
            no_demand,
            Arc::new(WorthQueryFixedExecutionCapacity::new("invalid-request", 1).unwrap()),
        ),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();
    assert_eq!(denial.kind(), &Kind::ResourceContract);
}
#[test]
fn atomic_boundary_and_physical_capacity_are_independent_authorities() {
    let atomic = atomic(Scale::selective(), Limits::selective());
    let denial = admit(&atomic, envelope(8)).unwrap_err();
    assert_eq!(denial.kind(), &Kind::ExecutionBoundaryUnsupported);
    let capacity = Arc::new(WorthQueryFixedExecutionCapacity::new("atomic-physical", 1).unwrap());
    let snapshot = support_with_capacity(atomic.clone(), capacity.clone());
    let plan = || {
        admit_execution_resource_plan(
            "binding",
            &declared(atomic.clone()),
            &atomic_request(&atomic),
            snapshot.clone(),
            WorthQueryExecutionResourceAdmissionCounters::default(),
        )
        .unwrap()
    };
    assert_eq!(
        plan().envelope().boundary(),
        WorthQueryExecutionBoundary::Atomic
    );
    let held = reserve_execution_resource_plan(plan()).unwrap();
    assert_eq!(capacity.active_attempts(), 1);
    assert!(reserve_execution_resource_plan(plan()).is_none());
    assert_eq!(capacity.active_attempts(), 1);
    drop(held);
    assert_eq!(capacity.active_attempts(), 0);
    let retry = reserve_execution_resource_plan(plan()).unwrap();
    assert_eq!(capacity.active_attempts(), 1);
    drop(retry);
    assert_eq!(capacity.active_attempts(), 0);
}
