use super::*;
use worth_query_admission::facade::resource_admission::{
    admit_execution_resource_plan, WorthQueryExecutionResourceAdmissionCounters,
    WorthQueryExecutionResourceAdmissionDenialKind, WorthQueryExecutionResourceSupportSnapshot,
};
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest;
use worth_query_installation::facade::{
    WorthQueryExecutionProviderRequirements, WorthQueryExecutionResourceContract,
    WorthQueryExecutionStrategyContract, WorthQueryExecutionStrategyName,
};

fn refusal(
    maximum_items: u64,
    maximum_batch: u64,
    items: u64,
    batch: u64,
) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionDenial {
    let profile =
        WorthQueryApplicationCandidateResourceProfile::physical_resources(maximum_items, 64)
            .unwrap()
            .with_maximum_operation_width(maximum_batch)
            .unwrap();
    let (support, capacity) = component_support(
        "independent-width",
        std::num::NonZeroUsize::new(1).unwrap(),
        profile,
    );
    assert_eq!(
        support
            .envelope()
            .optional_scale_ceiling(WorthQuerySemanticScaleAxis::CandidateItems),
        Some(maximum_items)
    );
    assert_eq!(
        support
            .envelope()
            .optional_scale_ceiling(WorthQuerySemanticScaleAxis::BatchWidth),
        Some(maximum_batch)
    );
    let required = WorthQueryExecutionResourceEnvelope::atomic(
        WorthQuerySemanticScaleRequest::selective()
            .with(WorthQuerySemanticScaleAxis::CandidateItems, items)
            .with(WorthQuerySemanticScaleAxis::BatchWidth, batch),
        support.envelope().resource_ceilings().clone(),
        support.envelope().cancellation_safe_point().clone(),
    );
    let request = WorthQueryExecutionResourceRequest::atomic(
        required.scale_ceilings().clone(),
        required.resource_ceilings().clone(),
        required.cancellation_safe_point().clone(),
    )
    .unwrap();
    let contract =
        WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
            WorthQueryExecutionStrategyName::new("independent-width").unwrap(),
            required,
            WorthQueryExecutionProviderRequirements::new(
                support.provider().clone(),
                support.access_product().clone(),
                support.allocator().clone(),
            ),
        )])
        .unwrap();
    let denial = admit_execution_resource_plan(
        "binding",
        &contract,
        &request,
        WorthQueryExecutionResourceSupportSnapshot::new(support, vec![], vec![], vec![], None),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::Backpressured
    );
    assert_eq!(denial.counters().capacity_reservations, 0);
    assert_eq!(capacity.active_attempts(), 0);
    denial
}
#[test]
fn atomic_primary_support_preserves_independent_candidate_and_batch_limits() {
    let candidate = refusal(1, 4_096, 2, 4_096);
    assert!(candidate.detail().contains("CandidateItems=1"));
    let batch = refusal(4_096, 1, 4_096, 2);
    assert!(batch.detail().contains("BatchWidth=1"));
}
