use super::MilestoneNineOneCertificationBundle;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::live::LiveQueryFamily;
use crate::subscription::{
    admit_query_subscription, certify_query_subscription_activation,
    certify_query_subscription_scale_slope, declare_query_subscription,
    lower_query_subscription_to_bridge, prepare_subscription_activation,
    select_query_subscription_family, LiveQueryAdmissionArtifact, QuerySubscriptionAdmissionBudget,
    QuerySubscriptionBridgeLoweringBudget, QuerySubscriptionConstructionSource,
    QuerySubscriptionRelationshipProofPosture, QuerySubscriptionScaleCounterSnapshot,
    QuerySubscriptionScaleFixtureSize, QuerySubscriptionSliceBudget, QuerySubscriptionWorkBudget,
};
use crate::view_shape_live::LiveViewShapeFamily;

pub(super) fn certified_lane(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    construction_source: QuerySubscriptionConstructionSource,
) -> Result<MilestoneNineOneCertificationBundle, IdentityDerivationDenial> {
    Ok(certified_lane_with_scale(
        live_family,
        view_family,
        construction_source,
        [10, 100, 1000],
    )?)
}

pub(super) fn certified_lane_with_basis(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    construction_source: QuerySubscriptionConstructionSource,
    basis_posture: crate::subscription::QuerySubscriptionBasisPosture,
) -> Result<MilestoneNineOneCertificationBundle, IdentityDerivationDenial> {
    Ok(certified_lane_from_live(
        LiveQueryAdmissionArtifact::for_test_with_basis(
            live_family,
            view_family,
            construction_source,
            basis_posture,
        )?,
        [10, 100, 1000],
    )?)
}

pub(super) fn certified_lane_with_scale(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    construction_source: QuerySubscriptionConstructionSource,
    row_counts: [u64; 3],
) -> Result<MilestoneNineOneCertificationBundle, IdentityDerivationDenial> {
    Ok(certified_lane_from_live(
        LiveQueryAdmissionArtifact::for_test(live_family, view_family, construction_source)?,
        row_counts,
    )?)
}

pub(super) fn certified_lane_with_context(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    construction_source: QuerySubscriptionConstructionSource,
    policy_digest: &str,
    tenant_digest: &str,
    relationship_proof_digest: &str,
) -> Result<MilestoneNineOneCertificationBundle, IdentityDerivationDenial> {
    Ok(certified_lane_from_live(
        LiveQueryAdmissionArtifact::for_test_with_context(
            live_family,
            view_family,
            construction_source,
            crate::subscription::QuerySubscriptionBasisPosture::CurrentHead,
            crate::subscription::QuerySubscriptionFutureSelection::ordinary()?,
            Some(policy_digest.to_string()),
            Some(tenant_digest.to_string()),
            Some(relationship_proof_digest.to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )?,
        [10, 100, 1000],
    )?)
}

fn certified_lane_from_live(
    live: LiveQueryAdmissionArtifact,
    row_counts: [u64; 3],
) -> Result<MilestoneNineOneCertificationBundle, IdentityDerivationDenial> {
    let query_digest = live.query_projection().label().to_string();
    let live_family_digest =
        digest_parts(&[format!("live_family:{}", live.live_family().as_str())]);
    let policy_digest = live.policy_projection().label().to_string();
    let tenant_basis_digest = live.tenant_projection().label().to_string();
    let relationship_proof_digest = live.relationship_proof_projection().label().to_string();
    let view_shape_digest = digest_parts(&[format!(
        "view_shape:{}",
        live.view_family()
            .map(|family| family.as_str())
            .unwrap_or("none")
    )]);
    let basis_digest = digest_parts(&[format!("basis:{}", live.basis_posture().as_str())]);
    let fixture_digest = digest_parts(&[
        format!("family:{}", live.live_family().as_str()),
        format!(
            "view:{}",
            live.view_family()
                .map(|family| family.as_str())
                .unwrap_or("none")
        ),
        format!("source:{}", live.construction_source().as_str()),
        format!("rows:{row_counts:?}"),
    ]);
    let selection = select_query_subscription_family(live, work_budget()).unwrap();
    let subscription_family_digest = digest_parts(&[format!(
        "subscription_family:{}",
        selection.family().as_str()
    )]);
    let subscription_equivalence_digest = selection
        .equivalence_basis()
        .equivalence_projection()
        .label()
        .to_string();
    let declaration = declare_query_subscription(selection, slice_budget()).unwrap();
    let query_family = declaration.family().as_str().to_string();
    let declaration_digest = declaration.declaration_projection().label().to_string();
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget()).unwrap();
    let bridge_family = lowering.bridge_family().as_str().to_string();
    let bridge_declaration_digest = lowering.bridge_declaration_projection().label().to_string();
    let basis_request_digest = lowering
        .basis_request()
        .basis_binding_projection()
        .label()
        .to_string();
    let signal_strategy_digest = lowering
        .signal_strategy_request()
        .signal_strategy_projection()
        .label()
        .to_string();
    let admission = admit_query_subscription(lowering, admission_budget()).unwrap();
    let support_profile_digest = admission
        .support_profile()
        .profile_projection()
        .label()
        .to_string();
    let diagnostics_digest = admission
        .diagnostics()
        .diagnostics_projection()
        .label()
        .to_string();
    let support_matrix_digest = digest_parts(&[
        format!("support:{support_profile_digest}"),
        format!("diagnostics:{diagnostics_digest}"),
    ]);
    let activation = prepare_subscription_activation(admission.clone());
    let scale_report = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            row_counts[0],
            &activation,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            row_counts[1],
            &activation,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            row_counts[2],
            &activation,
        )?,
    )
    .unwrap();
    let certification =
        certify_query_subscription_activation(admission, activation, scale_report).unwrap();

    Ok(MilestoneNineOneCertificationBundle {
        query_digest,
        live_family_digest,
        subscription_family_digest,
        subscription_equivalence_digest,
        policy_digest,
        tenant_basis_digest,
        relationship_proof_digest,
        view_shape_digest,
        basis_digest,
        query_family,
        bridge_family,
        basis_request_digest,
        signal_strategy_digest,
        declaration_digest,
        bridge_declaration_digest,
        admission_digest: certification.admission_projection().label().to_string(),
        activation_digest: certification.activation_projection().label().to_string(),
        certification_bundle_digest: certification
            .certification_bundle_projection()
            .label()
            .to_string(),
        support_profile_digest,
        diagnostics_digest,
        scale_slope_digest: certification.scale_slope_projection().label().to_string(),
        scale_activation_digest: certification
            .scale_activation_projection()
            .label()
            .to_string(),
        scale_admission_digest: certification
            .scale_admission_projection()
            .label()
            .to_string(),
        counter_snapshot_digest: digest_parts(&[
            format!(
                "admission_counters:{}",
                certification.admission_counter_projection().label()
            ),
            format!(
                "activation_counters:{}",
                certification.activation_counter_projection().label()
            ),
        ]),
        fixture_digest,
        compile_fail_boundary_digest: super::bundle_projection::compile_fail_boundary_digest(),
        support_matrix_digest,
    })
}

pub(super) fn admitted_activation(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    construction_source: QuerySubscriptionConstructionSource,
) -> Result<
    (
        crate::subscription::QuerySubscriptionAdmissionArtifact,
        crate::subscription::SubscriptionActivationInput,
    ),
    IdentityDerivationDenial,
> {
    let live = LiveQueryAdmissionArtifact::for_test(live_family, view_family, construction_source)?;
    let selection = select_query_subscription_family(live, work_budget()).unwrap();
    let declaration = declare_query_subscription(selection, slice_budget()).unwrap();
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget()).unwrap();
    let admission = admit_query_subscription(lowering, admission_budget()).unwrap();
    let activation = prepare_subscription_activation(admission.clone());
    Ok((admission, activation))
}

pub(super) fn work_budget() -> QuerySubscriptionWorkBudget {
    QuerySubscriptionWorkBudget::scratch_buffer_only(8, 8, 8, 64, 1)
}

pub(super) fn slice_budget() -> QuerySubscriptionSliceBudget {
    QuerySubscriptionSliceBudget::scratch_buffer_only(8, 8, 8, 8, 8, 8, 8, 8)
}

pub(super) fn lowering_budget() -> QuerySubscriptionBridgeLoweringBudget {
    QuerySubscriptionBridgeLoweringBudget::admitted(1, 8, 8, 1, 1)
}

pub(super) fn admission_budget() -> QuerySubscriptionAdmissionBudget {
    QuerySubscriptionAdmissionBudget::admitted(1, 1, 1, 1, 1)
}
