use crate::facade::{
    BridgeSubscriptionReferenceWorkloadComponentIdSet,
    BridgeSubscriptionReferenceWorkloadFamilyKind, BridgeSubscriptionReferenceWorkloadLaneIdSet,
    BridgeSubscriptionReferenceWorkloadLaneKind, BridgeSubscriptionReferenceWorkloadLaneRequest,
    BridgeSubscriptionReferenceWorkloadProductIdSet,
    BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest, RuntimeBridge,
};

use super::super::*;

pub(crate) fn temporal_async_closeout_request(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest {
    temporal_async_closeout_request_with_seed(runtime, "a", resource_request)
}

pub(crate) fn temporal_async_closeout_request_with_seed(
    runtime: &RuntimeBridge,
    seed: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest {
    let equivalent = temporal_async_bundle_equivalent_comparison(seed, resource_request);
    let diagnostics_delta =
        temporal_async_bundle_diagnostics_delta_comparison(seed, resource_request);
    let divergent = temporal_async_bundle_divergent_comparison(seed, resource_request);

    BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest::new(
        runtime.certify_subscription_certification_cost_posture(),
        runtime.certify_subscription_certification_schema_parity(),
        runtime.certify_subscription_certification_multi_failure_precedence(),
        runtime.certify_subscription_certification_ordering_hostility(),
        runtime.certify_subscription_certification_stale_checkpoint(),
        runtime.certify_subscription_certification_bundle_insufficiency(),
        runtime.certify_subscription_certification_historical_basis(),
        runtime.certify_subscription_certification_strategy_lowering(),
        runtime.certify_subscription_certification_fanout(),
        runtime.certify_subscription_certification_denied_continuation(),
        equivalent,
        diagnostics_delta,
        divergent,
        local_reference_workload_sufficiency(runtime),
    )
}

pub(crate) fn divergent_closeout_request(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest {
    let divergent = temporal_async_bundle_divergent_comparison("incomplete", resource_request);

    BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest::new(
        runtime.certify_subscription_certification_cost_posture(),
        runtime.certify_subscription_certification_schema_parity(),
        runtime.certify_subscription_certification_multi_failure_precedence(),
        runtime.certify_subscription_certification_ordering_hostility(),
        runtime.certify_subscription_certification_stale_checkpoint(),
        runtime.certify_subscription_certification_bundle_insufficiency(),
        runtime.certify_subscription_certification_historical_basis(),
        runtime.certify_subscription_certification_strategy_lowering(),
        runtime.certify_subscription_certification_fanout(),
        runtime.certify_subscription_certification_denied_continuation(),
        divergent.clone(),
        divergent.clone(),
        divergent,
        local_reference_workload_sufficiency(runtime),
    )
}

fn local_reference_workload_sufficiency(
    runtime: &RuntimeBridge,
) -> crate::facade::BridgeSubscriptionReferenceWorkloadSufficiency {
    let manifest = runtime
        .declare_subscription_reference_workload_manifest(
            BridgeSubscriptionReferenceWorkloadProductIdSet::from_declared_product_labels(
                (0..128).map(|slot| format!("product-{slot:03}")),
            ),
            BridgeSubscriptionReferenceWorkloadComponentIdSet::from_declared_component_labels([
                "steel", "rubber", "copper", "glass", "labor",
            ]),
            BridgeSubscriptionReferenceWorkloadLaneIdSet::from_declared_lane_labels([
                "authoritative-live",
                "time-only-routing",
                "historical-replay",
                "historical-basis-replay",
                "branch-local",
                "shared-fanout",
                "divergent-sharing-rejection",
                "stale-checkpoint-rejection",
                "restart-resume",
                "continuation",
                "denied-continuation",
                "preview-discard",
                "preview-promotion",
                "hostile-adapter-variation",
                "diagnostics-tier-variation",
                "canonical-ordering-hostility",
                "strategy-lowering-provenance",
                "bundle-insufficiency",
            ]),
        )
        .expect("reference workload manifest should seal");
    let declaration = runtime
        .plan_subscription_reference_workload(&manifest, reference_workload_lane_requests())
        .expect("reference workload declaration should admit");
    let lane_artifact_set = runtime
        .admit_subscription_reference_workload_lane_artifacts(&manifest, &declaration)
        .expect("reference workload lane artifacts should admit");
    let coverage_proof = runtime
        .prove_subscription_reference_workload_coverage(lane_artifact_set.clone())
        .expect("reference workload coverage should prove");
    runtime.seal_subscription_reference_workload_sufficiency(
        &manifest,
        &declaration,
        lane_artifact_set,
        &coverage_proof,
        "phase18-fixture-evidence",
    )
}

fn reference_workload_lane_requests() -> Vec<BridgeSubscriptionReferenceWorkloadLaneRequest> {
    vec![
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::HostileAdapterVariation,
            BridgeSubscriptionReferenceWorkloadFamilyKind::CollectionMembership,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::TimeOnlyRouting,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::HistoricalReplay,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::HistoricalBasisReplay,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::BranchLocal,
            BridgeSubscriptionReferenceWorkloadFamilyKind::CollectionMembership,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::SharedFanout,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::DivergentSharingRejection,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::StaleCheckpointRejection,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::RestartResume,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::Continuation,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::DeniedContinuation,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::PreviewDiscard,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::PreviewPromotion,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::AuthoritativeLive,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::DiagnosticsTierVariation,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::CanonicalOrderingHostility,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::StrategyLoweringProvenance,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
        BridgeSubscriptionReferenceWorkloadLaneRequest::new(
            BridgeSubscriptionReferenceWorkloadLaneKind::BundleInsufficiency,
            BridgeSubscriptionReferenceWorkloadFamilyKind::DetailExact,
        ),
    ]
}
