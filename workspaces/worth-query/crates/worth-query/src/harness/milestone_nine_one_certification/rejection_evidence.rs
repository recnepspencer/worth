use super::lane_builders::{
    admission_budget, admitted_activation, lowering_budget, slice_budget, work_budget,
};
use super::{MilestoneNineOneFailureClass, MilestoneNineOneRejectionBundle};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::live::LiveQueryFamily;
use crate::subscription::{
    admit_query_subscription, certify_query_subscription_activation,
    certify_query_subscription_scale_slope, declare_query_subscription,
    lower_query_subscription_to_bridge, select_query_subscription_family,
    LiveQueryAdmissionArtifact, QuerySubscriptionCertificationDenialKind,
    QuerySubscriptionConstructionSource, QuerySubscriptionDiagnosticStage,
    QuerySubscriptionRelationshipProofPosture, QuerySubscriptionScaleCounterSnapshot,
    QuerySubscriptionScaleFixtureSize,
};
use crate::view_shape_live::LiveViewShapeFamily;

pub(super) fn view_family_mismatch_rejection(
) -> Result<MilestoneNineOneRejectionBundle, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::Direct,
    )?;
    let error = select_query_subscription_family(live, work_budget()).unwrap_err();
    Ok(rejection(
        MilestoneNineOneFailureClass::FamilySelectionDenied,
        error.failure_class().as_str(),
        error.diagnostic()?.stage().as_str(),
        error.diagnostic()?.evidence_projection().label(),
        "",
        &[
            format!("message:{}", error.message()),
            format!(
                "diagnostic:{}",
                error.diagnostic()?.evidence_projection().label()
            ),
            format!("counters:{}", error.counters().counter_projection().label()),
        ],
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn bridge_family_rejection(
) -> Result<MilestoneNineOneRejectionBundle, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::Direct,
    )?;
    let selection = select_query_subscription_family(live, work_budget()).unwrap();
    let declaration = declare_query_subscription(selection, slice_budget()).unwrap();
    let error = lower_query_subscription_to_bridge(
        declaration,
        lowering_budget().without_bridge_family_support(),
    )
    .unwrap_err();
    Ok(rejection(
        MilestoneNineOneFailureClass::BridgeLoweringDenied,
        error.denial_kind().as_str(),
        error.diagnostic()?.stage().as_str(),
        error.diagnostic()?.evidence_projection().label(),
        "",
        &[
            format!("message:{}", error.message()),
            format!(
                "diagnostic:{}",
                error.diagnostic()?.evidence_projection().label()
            ),
            format!("counters:{}", error.counters().counter_projection().label()),
        ],
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn masked_slice_rejection(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
) -> Result<MilestoneNineOneRejectionBundle, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::Direct,
    )?;
    let selection = select_query_subscription_family(live, work_budget()).unwrap();
    let error = declare_query_subscription(
        selection,
        slice_budget().with_masked_slice_request_detected(),
    )
    .unwrap_err();
    Ok(rejection(
        MilestoneNineOneFailureClass::DeclarationDenied,
        error.denial_kind().as_str(),
        error.diagnostic()?.stage().as_str(),
        error.diagnostic()?.evidence_projection().label(),
        "",
        &[
            format!("message:{}", error.message()),
            format!(
                "diagnostic:{}",
                error.diagnostic()?.evidence_projection().label()
            ),
            format!("counters:{}", error.counters().counter_projection().label()),
        ],
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn broken_relationship_proof_rejection(
) -> Result<MilestoneNineOneRejectionBundle, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test_with_relationship_proof_posture(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::SavedExactReuse,
        QuerySubscriptionRelationshipProofPosture::Drifted,
    )?;
    let error = select_query_subscription_family(live, work_budget()).unwrap_err();
    Ok(rejection(
        MilestoneNineOneFailureClass::FamilySelectionDenied,
        error.failure_class().as_str(),
        error.diagnostic()?.stage().as_str(),
        error.diagnostic()?.evidence_projection().label(),
        "",
        &[
            format!("message:{}", error.message()),
            format!(
                "diagnostic:{}",
                error.diagnostic()?.evidence_projection().label()
            ),
            format!("counters:{}", error.counters().counter_projection().label()),
        ],
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn durable_reload_rejection(
) -> Result<MilestoneNineOneRejectionBundle, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
    )?;
    let selection = select_query_subscription_family(live, work_budget()).unwrap();
    let declaration = declare_query_subscription(selection, slice_budget()).unwrap();
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget()).unwrap();
    let error =
        admit_query_subscription(lowering, admission_budget().with_durable_reload_request())
            .unwrap_err();
    Ok(rejection(
        MilestoneNineOneFailureClass::AdmissionDenied,
        error.denial_kind().as_str(),
        error.pipeline_diagnostic().stage().as_str(),
        error.pipeline_diagnostic().evidence_projection().label(),
        error.support_profile().profile_projection().label(),
        &[
            format!("message:{}", error.message()),
            format!(
                "diagnostics:{}",
                error.diagnostics().diagnostics_projection().label()
            ),
            format!(
                "pipeline_diagnostic:{}",
                error.pipeline_diagnostic().evidence_projection().label()
            ),
            format!(
                "support:{}",
                error.support_profile().profile_projection().label()
            ),
            format!("counters:{}", error.counters().counter_projection().label()),
        ],
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn scale_source_mismatch_rejection(
) -> Result<MilestoneNineOneRejectionBundle, IdentityDerivationDenial> {
    let source = admitted_activation(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
    )?;
    let foreign = admitted_activation(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::Direct,
    )?;
    let foreign_scale = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            10,
            &foreign.1,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            100,
            &foreign.1,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            1000,
            &foreign.1,
        )?,
    )
    .unwrap();
    let error =
        certify_query_subscription_activation(source.0, source.1, foreign_scale).unwrap_err();
    debug_assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionCertificationDenialKind::ScaleSlopeSourceMismatch
    );
    Ok(rejection(
        MilestoneNineOneFailureClass::CertificationDenied,
        error.denial_kind().as_str(),
        QuerySubscriptionDiagnosticStage::Certification.as_str(),
        error.failure_projection()?.label(),
        "",
        &[
            format!("message:{}", error.message()),
            format!("failure:{}", error.failure_projection()?.label()),
        ],
        error.failure_projection()?.label().to_string(),
    ))
}

pub(super) fn scale_zero_row_rejection(
) -> Result<MilestoneNineOneRejectionBundle, IdentityDerivationDenial> {
    let (_, activation) = admitted_activation(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::Direct,
    )?;
    let error = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            0,
            &activation,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            10,
            &activation,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            100,
            &activation,
        )?,
    )
    .unwrap_err();
    Ok(rejection(
        MilestoneNineOneFailureClass::CertificationDenied,
        error.denial_kind().as_str(),
        QuerySubscriptionDiagnosticStage::Certification.as_str(),
        error.failure_projection()?.label(),
        "",
        &[
            format!("message:{}", error.message()),
            format!("failure:{}", error.failure_projection()?.label()),
        ],
        error.failure_projection()?.label().to_string(),
    ))
}

fn rejection(
    failure_class: MilestoneNineOneFailureClass,
    failure_kind: &str,
    diagnostic_stage: &str,
    diagnostic_digest: &str,
    support_profile_digest: &str,
    evidence_parts: &[String],
    counter_snapshot_digest: String,
) -> MilestoneNineOneRejectionBundle {
    let mut parts = vec![
        "milestone_nine_one_rejection_v1".to_string(),
        format!("failure_class:{failure_class:?}"),
        format!("failure_kind:{failure_kind}"),
        format!("diagnostic_stage:{diagnostic_stage}"),
        format!("diagnostic:{diagnostic_digest}"),
        format!("support:{support_profile_digest}"),
    ];
    parts.extend(evidence_parts.iter().cloned());
    MilestoneNineOneRejectionBundle {
        failure_class,
        failure_kind: failure_kind.to_string(),
        diagnostic_stage: diagnostic_stage.to_string(),
        diagnostic_digest: diagnostic_digest.to_string(),
        support_profile_digest: support_profile_digest.to_string(),
        failure_digest: digest_parts(&parts),
        counter_snapshot_digest,
    }
}
