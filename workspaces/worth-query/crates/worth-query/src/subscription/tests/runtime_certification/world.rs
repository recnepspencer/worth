use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

pub(super) struct RuntimeCertificationArtifacts {
    pub(super) selection: QuerySubscriptionFamilySelection,
    pub(super) declaration: QuerySubscriptionDeclarationArtifact,
    pub(super) lowering: BridgeSubscriptionLoweringPlan,
    pub(super) admission: QuerySubscriptionAdmissionArtifact,
    pub(super) support_report: QuerySubscriptionSupportReport,
    pub(super) parity_explanation: QuerySubscriptionBridgeParityExplanation,
    pub(super) lifecycle_bundle: SubscriptionLifecycleCertificationBundle,
    pub(super) admitted_bundle: QuerySubscriptionAdmittedDiagnosticBundle,
}

pub(super) struct HostileCoverageArtifacts {
    pub(super) denied_bundle: QuerySubscriptionDeniedDiagnosticBundle,
    pub(super) failure: QuerySubscriptionDiagnosticFailure,
}

pub(super) fn runtime_certification_artifacts_for(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
) -> RuntimeCertificationArtifacts {
    runtime_certification_artifacts_for_source(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
}

pub(super) fn runtime_certification_artifacts_for_source(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    source: QuerySubscriptionConstructionSource,
) -> RuntimeCertificationArtifacts {
    let live = LiveQueryAdmissionArtifact::for_test(live_family, view_family, source)
        .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live.clone(), roomy_budget()).unwrap();
    let context = SubscriptionLifecycleCertificationContext::from_live_selection(&live, &selection)
        .expect("fixture digest must fit");
    let declaration = declare_query_subscription(selection.clone(), roomy_slice_budget()).unwrap();
    let lowering =
        lower_query_subscription_to_bridge(declaration.clone(), roomy_lowering_budget()).unwrap();
    let admission = admit_query_subscription(
        lowering.clone(),
        QuerySubscriptionAdmissionBudget::admitted(1, 1, 1, 1, 1),
    )
    .unwrap();
    let activation = prepare_subscription_activation(admission.clone());
    let witness =
        build_query_subscription_manual_bridge_witness(&declaration, &lowering, &activation)
            .unwrap();
    let parity_explanation =
        explain_query_subscription_bridge_parity(&declaration, &lowering, &activation, witness)
            .unwrap()
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
    .unwrap();
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
    .unwrap();
    let handle = open_active_subscription_lane(&mut runtime, active_admission.clone()).unwrap();
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
    .unwrap();
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
    .unwrap();
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "affected-scope",
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let _delivery_window_digest = window.delivery_window_projection().label().to_string();
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta).unwrap();
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
    .unwrap();
    let delivery_batch =
        emit_query_delivery_batch(&mut runtime, window, work_packet.clone()).unwrap();
    let acknowledged_attachment = advance_subscription_acknowledgement(
        &mut runtime,
        attachment.clone(),
        delivery_batch.receipt().clone(),
    )
    .unwrap();
    let closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::TerminateConsumer(acknowledged_attachment.clone()),
    )
    .unwrap();
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
    .unwrap();
    let support_report = report_query_subscription_support(
        QuerySubscriptionSupportSubject::active_lifecycle(
            &declaration,
            &admission,
            &active_admission,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionSupportEvidence::admission(&declaration, &admission).unwrap(),
    )
    .unwrap()
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
    .unwrap();
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
    .unwrap()
    .0;

    RuntimeCertificationArtifacts {
        selection,
        declaration,
        lowering,
        admission,
        support_report,
        parity_explanation,
        lifecycle_bundle,
        admitted_bundle,
    }
}

pub(super) fn hostile_support_failure_for(
    artifacts: &RuntimeCertificationArtifacts,
) -> HostileCoverageArtifacts {
    let failure = QuerySubscriptionDiagnosticFailure::from_support_report_error(
        &report_query_subscription_support(
            QuerySubscriptionSupportSubject::activation(
                &artifacts.declaration,
                &prepare_subscription_activation(artifacts.admission.clone()),
            )
            .expect("fixture digest must fit"),
            QuerySubscriptionSupportEvidence::declaration(&artifacts.declaration),
        )
        .unwrap_err(),
    )
    .expect("fixture digest must fit");
    let selection_context =
        QuerySubscriptionDiagnosticSelectionContext::from_selection(&artifacts.selection);
    let denied_trace = trace_denied_query_subscription_diagnostics(
        &selection_context,
        Some(&artifacts.declaration),
        Some(&artifacts.lowering),
        Some(&artifacts.admission),
        None,
        failure.clone(),
    )
    .unwrap();
    let denied_bundle = bundle_denied_query_subscription_diagnostics(
        denied_trace,
        &selection_context,
        Some(&artifacts.declaration),
        Some(&artifacts.lowering),
        Some(&artifacts.admission),
        None,
        failure.clone(),
    )
    .unwrap()
    .0;

    HostileCoverageArtifacts {
        denied_bundle,
        failure,
    }
}
