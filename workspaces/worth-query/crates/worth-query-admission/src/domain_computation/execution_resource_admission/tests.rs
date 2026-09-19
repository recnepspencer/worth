mod capacity_diagnostics;
use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionDegradation, WorthQueryExecutionMode,
    WorthQueryExecutionResourceRequest, WorthQueryPartialEffectPosture,
    WorthQueryResourceDimension, WorthQueryResourceLimitRequest, WorthQueryRetainedProgressPosture,
    WorthQuerySemanticScaleAxis, WorthQuerySemanticScaleRequest, WorthQueryYieldedStatePosture,
};
use worth_query_installation::facade::{
    WorthQueryExecutionAccessProductFamily, WorthQueryExecutionAllocatorFamily,
    WorthQueryExecutionProviderFamily, WorthQueryExecutionProviderRequirements,
    WorthQueryExecutionResourceContract, WorthQueryExecutionResourceEnvelope,
    WorthQueryExecutionStrategyContract, WorthQueryExecutionStrategyName,
};

use super::*;

fn safe_point() -> WorthQueryCancellationSafePointFamily {
    WorthQueryCancellationSafePointFamily::new("admission-chunk").unwrap()
}

pub(super) fn envelope(limit: u64) -> WorthQueryExecutionResourceEnvelope {
    WorthQueryExecutionResourceEnvelope::new(
        WorthQuerySemanticScaleRequest::bounded(limit),
        WorthQueryResourceLimitRequest::bounded(limit),
        WorthQueryExecutionMode::Synchronous,
        None,
        safe_point(),
    )
}

pub(super) fn provider() -> WorthQueryExecutionProviderFamily {
    WorthQueryExecutionProviderFamily::new("admission-provider").unwrap()
}

pub(super) fn access() -> WorthQueryExecutionAccessProductFamily {
    WorthQueryExecutionAccessProductFamily::new("admission-access").unwrap()
}

pub(super) fn allocator() -> WorthQueryExecutionAllocatorFamily {
    WorthQueryExecutionAllocatorFamily::new("admission-arena").unwrap()
}

pub(super) fn contract(limit: u64) -> WorthQueryExecutionResourceContract {
    WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new("bounded").unwrap(),
        envelope(limit),
        WorthQueryExecutionProviderRequirements::new(provider(), access(), allocator()),
    )])
    .unwrap()
}

fn support(limit: u64) -> WorthQueryExecutionResourceSupportSnapshot {
    WorthQueryExecutionResourceSupportSnapshot::new(
        WorthQueryExecutionResourceSupport::new(
            provider(),
            access(),
            allocator(),
            envelope(limit),
            std::sync::Arc::new(
                WorthQueryFixedExecutionCapacity::mint("admission-test", 8).unwrap(),
            ),
        ),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
    )
}

pub(super) fn request(limit: u64) -> WorthQueryExecutionResourceRequest {
    WorthQueryExecutionResourceRequest::bounded(limit, limit, safe_point())
}

#[test]
fn exact_support_mints_one_resource_admission_plan() {
    let plan = admit_execution_resource_plan(
        "binding",
        &contract(8),
        &request(8),
        support(8),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap();

    assert_eq!(
        plan.posture(),
        WorthQueryExecutionResourceAdmissionPosture::Exact
    );
    assert_eq!(plan.counters().resource_contract_lookups, 1);
    assert_eq!(plan.counters().support_snapshot_checks, 1);
    assert_eq!(plan.counters().strategy_checks, 1);
}

#[test]
fn over_budget_request_denies_before_support_inspection() {
    let denial = admit_execution_resource_plan(
        "binding",
        &contract(8),
        &request(9),
        support(8),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::ResourceCeilingExceeded
    );
    assert_eq!(denial.counters().support_snapshot_checks, 0);
}

#[test]
fn provider_mismatch_is_distinct_from_capacity_denial() {
    let mismatched = WorthQueryExecutionResourceSupportSnapshot::new(
        WorthQueryExecutionResourceSupport::new(
            WorthQueryExecutionProviderFamily::new("foreign-provider").unwrap(),
            access(),
            allocator(),
            envelope(8),
            std::sync::Arc::new(
                WorthQueryFixedExecutionCapacity::mint("foreign-admission-test", 8).unwrap(),
            ),
        ),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
    );
    let denial = admit_execution_resource_plan(
        "binding",
        &contract(8),
        &request(8),
        mismatched,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::DifferentProviderRequired
    );
    assert_eq!(denial.counters().support_snapshot_checks, 1);
}

#[test]
fn partial_effect_posture_is_admitted_independently_from_degradation() {
    let partial = envelope(8)
        .with_partial_effect_posture(WorthQueryPartialEffectPosture::PartialEffectsMayRemain);
    let contract =
        WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
            WorthQueryExecutionStrategyName::new("partial-effects").unwrap(),
            partial.clone(),
            WorthQueryExecutionProviderRequirements::new(provider(), access(), allocator()),
        )])
        .unwrap();
    let support = support_with_capacity(
        partial,
        std::sync::Arc::new(WorthQueryFixedExecutionCapacity::mint("partial-effect", 1).unwrap()),
    );

    let denial = admit_execution_resource_plan(
        "binding",
        &contract,
        &request(8),
        support.clone(),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::PartialEffectPostureUnsupported
    );

    let admitted = admit_execution_resource_plan(
        "binding",
        &contract,
        &request(8)
            .allow_partial_effect_posture(WorthQueryPartialEffectPosture::PartialEffectsMayRemain),
        support,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap();
    assert_eq!(
        admitted.envelope().partial_effect_posture(),
        WorthQueryPartialEffectPosture::PartialEffectsMayRemain
    );
}

#[test]
fn yield_and_retained_progress_postures_require_independent_consent() {
    let yieldable = envelope(8)
        .with_yielded_state_posture(WorthQueryYieldedStatePosture::ProviderCheckpoint)
        .with_retained_progress_posture(WorthQueryRetainedProgressPosture::RetainAttemptCapacity);
    let contract =
        WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
            WorthQueryExecutionStrategyName::new("yieldable").unwrap(),
            yieldable.clone(),
            WorthQueryExecutionProviderRequirements::new(provider(), access(), allocator()),
        )])
        .unwrap();
    let support = support_with_capacity(
        yieldable,
        std::sync::Arc::new(WorthQueryFixedExecutionCapacity::mint("yield", 1).unwrap()),
    );

    let yield_denial = admit_execution_resource_plan(
        "binding",
        &contract,
        &request(8),
        support.clone(),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();
    assert_eq!(
        yield_denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::YieldedStatePostureUnsupported
    );

    let retained_denial = admit_execution_resource_plan(
        "binding",
        &contract,
        &request(8).allow_yielded_state_posture(WorthQueryYieldedStatePosture::ProviderCheckpoint),
        support.clone(),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();
    assert_eq!(
        retained_denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::RetainedProgressPostureUnsupported
    );

    let admitted = admit_execution_resource_plan(
        "binding",
        &contract,
        &request(8)
            .allow_yielded_state_posture(WorthQueryYieldedStatePosture::ProviderCheckpoint)
            .allow_retained_progress_posture(
                WorthQueryRetainedProgressPosture::RetainAttemptCapacity,
            ),
        support,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap();
    assert_eq!(
        admitted.envelope().yielded_state_posture(),
        WorthQueryYieldedStatePosture::ProviderCheckpoint
    );
    assert_eq!(
        admitted.envelope().retained_progress_posture(),
        WorthQueryRetainedProgressPosture::RetainAttemptCapacity
    );
}

#[test]
fn partial_result_degradation_requires_explicit_request_consent() {
    let degradation = WorthQueryExecutionDegradation::PartialResult;
    let degraded = WorthQueryExecutionResourceEnvelope::new(
        WorthQuerySemanticScaleRequest::bounded(8),
        WorthQueryResourceLimitRequest::bounded(8),
        WorthQueryExecutionMode::Synchronous,
        Some(degradation),
        safe_point(),
    );
    let contract =
        WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
            WorthQueryExecutionStrategyName::new(degradation.as_str()).unwrap(),
            degraded.clone(),
            WorthQueryExecutionProviderRequirements::new(provider(), access(), allocator()),
        )])
        .unwrap();
    let support = support_with_capacity(
        degraded,
        std::sync::Arc::new(
            WorthQueryFixedExecutionCapacity::mint(degradation.as_str(), 1).unwrap(),
        ),
    );

    let denied = admit_execution_resource_plan(
        "binding",
        &contract,
        &request(8),
        support.clone(),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();
    assert_eq!(
        denied.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::DegradationPostureUnsupported
    );
    let admitted = admit_execution_resource_plan(
        "binding",
        &contract,
        &request(8).allow_degradation(degradation),
        support,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap();
    assert_eq!(admitted.envelope().degradation(), Some(degradation));
}

#[test]
fn one_axis_at_a_time_changes_only_the_resource_decision() {
    for axis in WorthQuerySemanticScaleAxis::ALL {
        let request = WorthQueryExecutionResourceRequest::new(
            WorthQuerySemanticScaleRequest::bounded(1).with(axis, 9),
            WorthQueryResourceLimitRequest::bounded(1),
            safe_point(),
        )
        .unwrap();
        assert_single_axis_denial(request);
    }
    for dimension in WorthQueryResourceDimension::ALL {
        let request = WorthQueryExecutionResourceRequest::new(
            WorthQuerySemanticScaleRequest::bounded(1),
            WorthQueryResourceLimitRequest::bounded(1).with(dimension, 9),
            safe_point(),
        )
        .unwrap();
        assert_single_axis_denial(request);
    }
}

fn assert_single_axis_denial(request: WorthQueryExecutionResourceRequest) {
    let denial = admit_execution_resource_plan(
        "binding",
        &contract(8),
        &request,
        support(8),
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        &WorthQueryExecutionResourceAdmissionDenialKind::ResourceCeilingExceeded
    );
    assert_eq!(denial.counters().strategy_checks, 1);
    assert_eq!(
        denial.counters().envelope_dimension_checks,
        WorthQuerySemanticScaleAxis::ALL.len() + WorthQueryResourceDimension::ALL.len()
    );
    assert_eq!(denial.counters().support_snapshot_checks, 0);
    assert_eq!(denial.counters().capacity_reservations, 0);
    assert_eq!(denial.counters().provider_session_mints, 0);
}

pub(super) fn support_with_capacity(
    resource_envelope: WorthQueryExecutionResourceEnvelope,
    capacity: std::sync::Arc<dyn WorthQueryExecutionCapacityPort>,
) -> WorthQueryExecutionResourceSupportSnapshot {
    WorthQueryExecutionResourceSupportSnapshot::new(
        WorthQueryExecutionResourceSupport::new(
            provider(),
            access(),
            allocator(),
            resource_envelope,
            capacity,
        ),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
    )
}
