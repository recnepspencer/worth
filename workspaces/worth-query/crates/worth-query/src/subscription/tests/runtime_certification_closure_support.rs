use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

pub(crate) struct RuntimeBackedSubscriptionCertificationSummary {
    pub certified_family_count: u64,
    pub hostile_row_coverage_count: u64,
    pub support_report_digest: String,
    pub bridge_parity_digest: String,
    pub diagnostic_bundle_digest: String,
    pub lifecycle_certification_digest: String,
    pub coverage_resolution_posture: CoverageResolutionPosture,
}

pub(crate) fn runtime_backed_subscription_certification_summary(
) -> RuntimeBackedSubscriptionCertificationSummary {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None::<LiveViewShapeFamily>,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(
        live.clone().expect("fixture digest must fit"),
        QuerySubscriptionWorkBudget::scratch_buffer_only(8, 8, 8, 32, 1),
    )
    .expect("runtime-backed detail family should select");
    let context = SubscriptionLifecycleCertificationContext::from_live_selection(&live, &selection)
        .expect("fixture digest must fit");
    let declaration = declare_query_subscription(
        selection.clone(),
        QuerySubscriptionSliceBudget::scratch_buffer_only(8, 8, 8, 8, 8, 8, 8, 8),
    )
    .expect("runtime-backed detail family should declare");
    let lowering = lower_query_subscription_to_bridge(
        declaration.clone(),
        QuerySubscriptionBridgeLoweringBudget::admitted(1, 8, 8, 1, 1),
    )
    .expect("runtime-backed detail family should lower");
    let admission = admit_query_subscription(
        lowering.clone(),
        QuerySubscriptionAdmissionBudget::admitted(1, 1, 1, 1, 1),
    )
    .expect("runtime-backed detail family should admit");
    let activation = prepare_subscription_activation(admission.clone());
    let witness =
        build_query_subscription_manual_bridge_witness(&declaration, &lowering, &activation)
            .expect("runtime-backed detail family should build manual bridge witness");
    let parity_explanation =
        explain_query_subscription_bridge_parity(&declaration, &lowering, &activation, witness)
            .expect("runtime-backed detail family should explain bridge parity")
            .0;
    let scale_report = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            10,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            100,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            1000,
            &activation,
        )
        .expect("fixture digest must fit"),
    )
    .expect("runtime-backed detail family should certify scale slope");
    let mut runtime = ActiveSubscriptionRuntime::new();
    let active_admission = admit_active_subscription_lane(
        activation.clone(),
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(1),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::LifecycleArena,
        ),
    )
    .expect("runtime-backed detail family should admit active lane");
    let handle = open_active_subscription_lane(&mut runtime, active_admission.clone())
        .expect("runtime-backed detail family should open active lane");
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("runtime-certification", "cursor")
            .expect("fixture digest must fit"),
        SubscriptionConsumerAttachmentBudget::admitted(
            ActiveFanoutWidth::measured(1),
            ConsumerDeliveryPacingWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            DeliveryBackpressurePolicy::RetainWithinWindow,
        ),
    )
    .expect("runtime-backed detail family should attach consumer");
    let window = open_query_delivery_window(
        &mut runtime,
        &attachment,
        QueryDeliveryWindowBudget::admitted(
            DeliveryWindowWidth::measured(3),
            PatchGroupWidth::measured(1),
            MaintenanceDeltaWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
            DeliveryBackpressurePolicy::RetainWithinWindow,
        ),
    )
    .expect("runtime-backed detail family should open delivery window");
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "affected-scope",
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let _delivery_window_digest = window.delivery_window_projection().label().to_string();
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta)
        .expect("runtime-backed detail family should lower maintenance delta");
    let work_packet = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        delta.clone(),
        lowering_report.clone(),
        ActiveDeliveryDensityPosture::SparseDelta,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(1),
        ActiveDeliveryContinuationWidth::measured(0),
        ActiveDeliveryPreviewResidueWidth::measured(0),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::PatchScratch,
    )
    .expect("runtime-backed detail family should build delivery work packet");
    let delivery_batch = emit_query_delivery_batch(&mut runtime, window, work_packet.clone())
        .expect("runtime-backed detail family should emit delivery batch");
    let acknowledged_attachment = advance_subscription_acknowledgement(
        &mut runtime,
        attachment.clone(),
        delivery_batch.receipt().clone(),
    )
    .expect("runtime-backed detail family should acknowledge delivery");
    let closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::TerminateConsumer(acknowledged_attachment.clone()),
    )
    .expect("runtime-backed detail family should close lifecycle");
    let lifecycle_bundle = certify_subscription_lifecycle(
        context,
        &admission,
        &activation,
        &scale_report,
        &active_admission,
        &handle,
        &attachment,
        delivery_batch.delivery_window_identity(),
        &delta,
        &lowering_report,
        &work_packet,
        &delivery_batch,
        &acknowledged_attachment,
        None,
        SubscriptionLifecyclePreviewCertification::None,
        &closeout,
    )
    .expect("runtime-backed detail family should certify lifecycle");
    let support_report = report_query_subscription_support(
        QuerySubscriptionSupportSubject::active_lifecycle(
            &declaration,
            &admission,
            &active_admission,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionSupportEvidence::admission(&declaration, &admission)
            .expect("runtime-backed detail family should admit support evidence"),
    )
    .expect("runtime-backed detail family should report support")
    .0;
    let admitted_trace = trace_admitted_query_subscription_diagnostics(
        &selection,
        &declaration,
        &lowering,
        &admission,
        &support_report,
        &lifecycle_bundle,
        None,
        None,
        Some(&closeout),
    )
    .expect("runtime-backed detail family should trace admitted diagnostics");
    let admitted_bundle = bundle_admitted_query_subscription_diagnostics(
        admitted_trace,
        &selection,
        &declaration,
        &lowering,
        &admission,
        &support_report,
        &lifecycle_bundle,
        None,
        None,
        Some(&closeout),
    )
    .expect("runtime-backed detail family should bundle admitted diagnostics")
    .0;

    let support_failure = QuerySubscriptionDiagnosticFailure::from_support_report_error(
        &report_query_subscription_support(
            QuerySubscriptionSupportSubject::activation(
                &declaration,
                &prepare_subscription_activation(admission.clone()),
            )
            .expect("fixture digest must fit"),
            QuerySubscriptionSupportEvidence::declaration(&declaration),
        )
        .expect_err("activation support should fail closed for hostile runtime coverage"),
    )
    .expect("fixture digest must fit");
    let selection_context = QuerySubscriptionDiagnosticSelectionContext::from_selection(&selection);
    let denied_trace = trace_denied_query_subscription_diagnostics(
        &selection_context,
        Some(&declaration),
        Some(&lowering),
        Some(&admission),
        None,
        support_failure.clone().expect("fixture digest must fit"),
    )
    .expect("runtime-backed detail family should trace denied diagnostics");
    let denied_bundle = bundle_denied_query_subscription_diagnostics(
        denied_trace,
        &selection_context,
        Some(&declaration),
        Some(&lowering),
        Some(&admission),
        None,
        support_failure.clone().expect("fixture digest must fit"),
    )
    .expect("runtime-backed detail family should bundle denied diagnostics")
    .0;

    let admitted_row = QuerySubscriptionFamilyCoverageRow::admitted(
        declaration.family(),
        &support_report,
        &parity_explanation,
        &lifecycle_bundle,
        &admitted_bundle,
        QuerySubscriptionLifecycleCoverageClass::LifecycleCloseout,
    )
    .expect("runtime-backed detail family should admit certification row");
    let hostile_row = QuerySubscriptionFamilyCoverageRow::hostile(
        declaration.family(),
        &support_report,
        &parity_explanation,
        &lifecycle_bundle,
        &denied_bundle,
        &support_failure,
        QuerySubscriptionLifecycleCoverageClass::ActiveLifecycle,
    )
    .expect("runtime-backed detail family should admit hostile certification row");
    let matrix = build_query_subscription_family_coverage_matrix(vec![admitted_row, hostile_row])
        .expect("fixture digest must fit");
    let handle = build_certified_family_coverage_handle(
        &matrix,
        declaration.family(),
        CoverageResolutionPosture::IndexedCoverageSet,
    )
    .expect("runtime-backed detail family should build certified coverage handle");
    let scope = build_query_subscription_runtime_certification_scope(
        support_report.clone(),
        parity_explanation.clone(),
        admitted_bundle.clone(),
        lifecycle_bundle.clone(),
        handle,
    )
    .expect("runtime-backed detail family should build runtime certification scope");
    let (bundle, receipt) = certify_query_subscription_runtime_family(scope)
        .expect("runtime-backed detail family should certify runtime family");

    RuntimeBackedSubscriptionCertificationSummary {
        certified_family_count: bundle.counters().certified_family_count(),
        hostile_row_coverage_count: bundle.counters().hostile_row_coverage_count(),
        support_report_digest: bundle.support_report_projection().label().to_string(),
        bridge_parity_digest: bundle.bridge_parity_projection().label().to_string(),
        diagnostic_bundle_digest: bundle.diagnostic_bundle_projection().label().to_string(),
        lifecycle_certification_digest: bundle
            .lifecycle_certification_projection()
            .label()
            .to_string(),
        coverage_resolution_posture: *receipt.coverage_resolution_posture(),
    }
}
