use super::evidence::denied_support_artifacts_from_parts;
use super::lane_fixtures::{
    active_budget, attachment_budget, continuation_harness_identity, delivery_budget, roomy_budget,
    roomy_lowering_budget, roomy_slice_budget,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::live::LiveQueryFamily;
use crate::subscription::*;
use crate::view_shape_live::LiveViewShapeFamily;

#[derive(Clone, Copy)]
pub(super) enum LaneScenario {
    ActiveLifecycle,
    Continuation,
    PreviewDiscard,
}

#[derive(Clone)]
pub(super) struct CertifiedLaneArtifacts {
    pub(super) selection: QuerySubscriptionFamilySelection,
    pub(super) declaration: QuerySubscriptionDeclarationArtifact,
    pub(super) lowering: BridgeSubscriptionLoweringPlan,
    pub(super) admission: QuerySubscriptionAdmissionArtifact,
    pub(super) support_report: QuerySubscriptionSupportReport,
    pub(super) support_lookup_receipt: SupportLookupReceipt,
    pub(super) witness: QuerySubscriptionManualBridgeWitness,
    pub(super) parity_explanation: QuerySubscriptionBridgeParityExplanation,
    pub(super) parity_receipt: BridgeParityReceipt,
    pub(super) lifecycle_bundle: SubscriptionLifecycleCertificationBundle,
    pub(super) admitted_trace: QuerySubscriptionDiagnosticTrace,
    pub(super) admitted_bundle: QuerySubscriptionAdmittedDiagnosticBundle,
    pub(super) diagnostic_receipt: DiagnosticAssemblyReceipt,
    pub(super) runtime_bundle: QuerySubscriptionRuntimeCertificationBundle,
    pub(super) coverage_receipt: CertificationCoverageReceipt,
    pub(super) continuation_digest: String,
    pub(super) preview_isolation_digest: String,
}

pub(super) fn lane_for(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    scenario: LaneScenario,
    coverage_posture: CoverageResolutionPosture,
    admission_width: usize,
) -> Result<CertifiedLaneArtifacts, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
    )?;
    let selection = select_query_subscription_family(live.clone(), roomy_budget()).unwrap();
    let context =
        SubscriptionLifecycleCertificationContext::from_live_selection(&live, &selection)?;
    let declaration = declare_query_subscription(selection.clone(), roomy_slice_budget()).unwrap();
    let lowering =
        lower_query_subscription_to_bridge(declaration.clone(), roomy_lowering_budget()).unwrap();
    let admission = admit_query_subscription(
        lowering.clone(),
        QuerySubscriptionAdmissionBudget::admitted(admission_width, 8, 1, 1, 1),
    )
    .unwrap();
    let activation = prepare_subscription_activation(admission.clone());
    let witness =
        build_query_subscription_manual_bridge_witness(&declaration, &lowering, &activation)
            .unwrap();
    let (parity_explanation, parity_receipt) = explain_query_subscription_bridge_parity(
        &declaration,
        &lowering,
        &activation,
        witness.clone(),
    )
    .unwrap();
    let scale_report = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            10,
            &activation,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            100,
            &activation,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            1000,
            &activation,
        )?,
    )
    .unwrap();

    let mut runtime = ActiveSubscriptionRuntime::new();
    let active_admission = admit_active_subscription_lane(activation, active_budget()).unwrap();
    let handle = open_active_subscription_lane(&mut runtime, active_admission.clone()).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("certification-consumer", "cursor")?,
        attachment_budget(),
    )
    .unwrap();
    let mut continuation_report = None;
    let mut preview_isolation = None;
    let mut preview_residue = None;
    let mut preview_discard = None;
    let (window, delta, lowering_report, continuation_width) = match scenario {
        LaneScenario::Continuation => {
            let window =
                open_query_delivery_window(&mut runtime, &attachment, delivery_budget()).unwrap();
            let evidence = admit_subscription_continuation_evidence(
                attachment.lane_digest().clone(),
                SubscriptionContinuationClass::IdentityRemap,
                continuation_harness_identity("employee:old")?,
                continuation_harness_identity("employee:new")?,
                continuation_harness_identity("basis:current")?,
                continuation_harness_identity("identity-evolution-authority")?,
                ContinuationRemapWidth::measured(1),
            )
            .unwrap();
            let (continued_window, report) =
                apply_active_subscription_continuation(&mut runtime, window, evidence).unwrap();
            let (delta, _) = lower_subscription_continuation_report(&report)?;
            let (delta, lowering_report, _) =
                lower_query_subscription_maintenance_delta(delta).unwrap();
            continuation_report = Some(report);
            (continued_window, delta, lowering_report, 1)
        }
        LaneScenario::ActiveLifecycle | LaneScenario::PreviewDiscard => {
            let window =
                open_query_delivery_window(&mut runtime, &attachment, delivery_budget()).unwrap();
            let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
                QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
                attachment.lane_digest().clone(),
                "affected-scope",
                MaintenanceDeltaWidth::measured(1),
            )?;
            let (delta, lowering_report, _) =
                lower_query_subscription_maintenance_delta(delta).unwrap();
            (window, delta, lowering_report, 0)
        }
    };

    if matches!(scenario, LaneScenario::PreviewDiscard) {
        let isolation = admit_preview_subscription_isolation(
            &attachment,
            "preview-epoch-a",
            PreviewResidueWidth::measured(2),
        )
        .unwrap();
        let residue = measure_preview_subscription_residue(
            PreviewResidueWidth::measured(0),
            PreviewResidueWidth::measured(0),
            PreviewResidueWidth::measured(0),
            PreviewResidueWidth::measured(0),
            PreviewResidueWidth::measured(0),
            PreviewResidueWidth::measured(1),
            PreviewResidueWidth::measured(1),
        )?;
        let discard = discard_preview_subscription(isolation.clone(), residue.clone()).unwrap();
        preview_isolation = Some(isolation);
        preview_residue = Some(residue);
        preview_discard = Some(discard);
    }

    let _delivery_window_digest = window.delivery_window_projection().label().to_string();
    let work_packet = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        delta.clone(),
        lowering_report.clone(),
        ActiveDeliveryDensityPosture::SparseDelta,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(1),
        ActiveDeliveryContinuationWidth::measured(continuation_width),
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
    let close_request = match &preview_discard {
        Some(discard) => SubscriptionLifecycleCloseRequest::PreviewDiscard(discard.clone()),
        None => {
            SubscriptionLifecycleCloseRequest::TerminateConsumer(acknowledged_attachment.clone())
        }
    };
    let closeout = close_subscription_lifecycle(&mut runtime, &handle, close_request).unwrap();
    let preview_certification = match (&preview_isolation, &preview_residue, &preview_discard) {
        (Some(isolation), Some(residue), Some(discard)) => {
            SubscriptionLifecyclePreviewCertification::Discard {
                isolation,
                residue_report: residue,
                discard_closeout: discard,
            }
        }
        _ => SubscriptionLifecyclePreviewCertification::None,
    };
    let lifecycle_bundle = certify_subscription_lifecycle(
        context,
        &admission,
        &prepare_subscription_activation(admission.clone()),
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
        continuation_report.as_ref(),
        preview_certification,
        &closeout,
    )
    .unwrap();
    let support_subject = match scenario {
        LaneScenario::ActiveLifecycle => QuerySubscriptionSupportSubject::active_lifecycle(
            &declaration,
            &admission,
            &active_admission,
        )?,
        LaneScenario::Continuation => QuerySubscriptionSupportSubject::continuation(
            &declaration,
            &admission,
            continuation_report.as_ref().unwrap(),
        )?,
        LaneScenario::PreviewDiscard => {
            QuerySubscriptionSupportSubject::preview_closeout(&declaration, &admission, &closeout)?
        }
    };
    let (support_report, support_lookup_receipt) = report_query_subscription_support(
        support_subject,
        QuerySubscriptionSupportEvidence::admission(&declaration, &admission).unwrap(),
    )
    .unwrap();
    let admitted_trace = trace_admitted_query_subscription_diagnostics(
        &selection,
        &declaration,
        &lowering,
        &admission,
        &support_report,
        &lifecycle_bundle,
        continuation_report.as_ref(),
        preview_isolation.as_ref(),
        Some(&closeout),
    )
    .unwrap();
    let (admitted_bundle, diagnostic_receipt) = bundle_admitted_query_subscription_diagnostics(
        admitted_trace.clone(),
        &selection,
        &declaration,
        &lowering,
        &admission,
        &support_report,
        &lifecycle_bundle,
        continuation_report.as_ref(),
        preview_isolation.as_ref(),
        Some(&closeout),
    )
    .unwrap();

    let hostile =
        denied_support_artifacts_from_parts(&selection, &declaration, &lowering, &admission)?;
    let matrix = build_query_subscription_family_coverage_matrix(vec![
        QuerySubscriptionFamilyCoverageRow::admitted(
            declaration.family(),
            &support_report,
            &parity_explanation,
            &lifecycle_bundle,
            &admitted_bundle,
            lifecycle_class_for(scenario),
        )
        .unwrap(),
        QuerySubscriptionFamilyCoverageRow::hostile(
            declaration.family(),
            &support_report,
            &parity_explanation,
            &lifecycle_bundle,
            &hostile.denied_bundle,
            &hostile.failure,
            QuerySubscriptionLifecycleCoverageClass::ActiveLifecycle,
        )
        .unwrap(),
    ])?;
    let handle =
        build_certified_family_coverage_handle(&matrix, declaration.family(), coverage_posture)
            .unwrap();
    let scope = build_query_subscription_runtime_certification_scope(
        support_report.clone(),
        parity_explanation.clone(),
        admitted_bundle.clone(),
        lifecycle_bundle.clone(),
        handle,
    )
    .unwrap();
    let (runtime_bundle, coverage_receipt) =
        certify_query_subscription_runtime_family(scope).unwrap();

    Ok(CertifiedLaneArtifacts {
        selection,
        declaration,
        lowering,
        admission,
        support_report,
        support_lookup_receipt,
        witness,
        parity_explanation,
        parity_receipt,
        lifecycle_bundle,
        admitted_trace,
        admitted_bundle,
        diagnostic_receipt,
        runtime_bundle,
        coverage_receipt,
        continuation_digest: continuation_report
            .as_ref()
            .map(|value| value.report_projection().label().to_string())
            .unwrap_or_else(|| "none".to_string()),
        preview_isolation_digest: preview_isolation
            .as_ref()
            .map(|value| value.isolation_projection().label().to_string())
            .unwrap_or_else(|| "none".to_string()),
    })
}

pub(super) fn lifecycle_class_for(
    scenario: LaneScenario,
) -> QuerySubscriptionLifecycleCoverageClass {
    match scenario {
        LaneScenario::ActiveLifecycle => QuerySubscriptionLifecycleCoverageClass::ActiveLifecycle,
        LaneScenario::Continuation => QuerySubscriptionLifecycleCoverageClass::Continuation,
        LaneScenario::PreviewDiscard => QuerySubscriptionLifecycleCoverageClass::PreviewIsolation,
    }
}
