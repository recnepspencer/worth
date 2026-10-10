use super::*;

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_workload_lane_ids(
) -> BridgeSubscriptionReferenceWorkloadLaneIdSet {
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
    ])
}

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_workload_lane_requests(
) -> Vec<BridgeSubscriptionReferenceWorkloadLaneRequest> {
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

pub(in crate::harness::tests::pricing_shock) fn pricing_reference_workload_manifest(
    runtime: &RuntimeBridge,
) -> BridgeSubscriptionReferenceWorkloadManifestSealed {
    runtime
        .declare_subscription_reference_workload_manifest(
            BridgeSubscriptionReferenceWorkloadProductIdSet::from_declared_product_labels(
                (0..128).map(|slot| format!("product-{slot:03}")),
            ),
            BridgeSubscriptionReferenceWorkloadComponentIdSet::from_declared_component_labels([
                "steel", "rubber", "copper", "glass", "labor",
            ]),
            pricing_reference_workload_lane_ids(),
        )
        .expect("pricing reference workload manifest should seal")
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_reference_workload_sufficiency(
    policy: BridgeRuntimePolicy,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeSubscriptionReferenceWorkloadSufficiency {
    let fixture_bundle = capture_pricing_workload_certification_bundle(
        policy,
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-reference-workload-skin"),
        resource_request,
    );
    let runtime = build_pricing_runtime_with_policy(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        policy,
    );
    let manifest = pricing_reference_workload_manifest(&runtime);
    let declaration = runtime
        .plan_subscription_reference_workload(&manifest, pricing_reference_workload_lane_requests())
        .expect("pricing reference workload should admit a declared lane plan");
    let lane_artifact_set = runtime
        .admit_subscription_reference_workload_lane_artifacts(&manifest, &declaration)
        .expect("pricing reference workload should admit lane artifacts");
    let coverage_proof = runtime
        .prove_subscription_reference_workload_coverage(lane_artifact_set.clone())
        .expect("pricing reference workload should prove phase 17 coverage");
    runtime.seal_subscription_reference_workload_sufficiency(
        &manifest,
        &declaration,
        lane_artifact_set,
        &coverage_proof,
        &fixture_bundle.digest(),
    )
}
