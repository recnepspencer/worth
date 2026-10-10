use worth_runtime_bridge::facade::{BridgeTruthViewEvaluationRequest, TruthBranchIdentity};

use crate::basis_lifecycle::{
    basis_lifecycle, readmit_lower_runtime_evidence, LowerRuntimeBasisEvidence,
};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeReadmissionReceipt,
    WorthQueryLowerRuntimeRouteKind, WorthQueryLowerRuntimeRoutePlan,
    WorthQueryLowerRuntimeSeamKey,
};

use super::super::{RepresentativeArtifacts, WorthQueryLowerRuntimeRepresentativeEvidenceSource};
use super::readmission_support::{
    continuity_runtime, delivered_continuity, detail_subscription, observation_runtime,
    subscription_runtime, PHASE_SIX_MAIN_BRANCH,
};

pub(crate) fn representative_subscription_continuity_row(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> RepresentativeArtifacts {
    let runtime = continuity_runtime();
    let continuity = delivered_continuity(&runtime, resource_request);
    let record = continuity.canonical_record();
    let continuity_evidence =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("route_identity"),
                record
                    .route_identity()
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("continuity_identity"),
                continuity
                    .continuity_identity()
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("continuity_resolution"),
                record.continuity_resolution_digest(),
            )
            .seal();
    let scoped = basis_lifecycle()
        .runtime_snapshot("phase-six-continuity", "runtime-bridge-continuity")
        .inspect()
        .expect("continuity inspection basis should admit");
    readmit_lower_runtime_evidence(
        scoped,
        LowerRuntimeBasisEvidence::from_runtime_bridge_facade(
            "runtime-bridge-continuity",
            continuity_evidence.as_str(),
            1,
        ),
    )
    .expect("continuity fixture should use canonical scoped readmission");

    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::SubscriptionContinuity,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::RuntimeBridge,
        "Subscription continuity",
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "subscription-continuity-route-subject",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("continuity"),
            &continuity_evidence,
        )
        .seal(),
    );
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request.clone(),
        &continuity_evidence,
    );
    let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility.clone(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "subscription-continuity-route",
            &continuity_evidence,
        ),
    );
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "phase-six-continuity-route",
            &continuity_evidence,
        );
    let boundary_receipt = WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
        &route_plan,
        &retained_evidence,
    );
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        WorthQueryLowerRuntimeSeamKey::SubscriptionContinuity,
        &route_plan,
        &boundary_receipt,
        &retained_evidence,
    );
    RepresentativeArtifacts {
        seam_key: WorthQueryLowerRuntimeSeamKey::SubscriptionContinuity,
        request,
        eligibility,
        route_plan: Some(route_plan),
        boundary_receipt,
        envelope,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    }
}

pub(crate) fn representative_basis_truth_view_readmission_row(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> RepresentativeArtifacts {
    let runtime = observation_runtime();
    let evaluation = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                TruthBranchIdentity::from_relational_branch_id(PHASE_SIX_MAIN_BRANCH),
            ),
            resource_request,
        )
        .expect("truth-view readmission fixture should evaluate branch head");
    let selector = evaluation.record().declaration().selector();
    let truth_view_evidence =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("record_identity"),
                evaluation
                    .record()
                    .record_identity()
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("selector_identity"),
                selector
                    .selector_identity()
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority"),
                evaluation.record().decision_log().authority_digest(),
            )
            .seal();
    let scoped = basis_lifecycle()
        .runtime_snapshot("phase-six-truth-view", "runtime-bridge-truth-view")
        .observe()
        .expect("truth-view observation basis should admit");
    readmit_lower_runtime_evidence(
        scoped,
        LowerRuntimeBasisEvidence::from_runtime_bridge_facade(
            "runtime-bridge-truth-view",
            truth_view_evidence.as_str(),
            1,
        ),
    )
    .expect("truth-view fixture should use canonical scoped readmission");

    readmission_row(
        WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromTruthViewEvidence,
        "Basis readmission from truth-view evidence",
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "basis-truth-view-readmission-subject",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("truth_view"),
            &truth_view_evidence,
        )
        .seal(),
        truth_view_evidence.clone(),
        truth_view_evidence,
    )
}

pub(crate) fn representative_basis_subscription_readmission_row(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> RepresentativeArtifacts {
    let runtime = subscription_runtime();
    let admitted = detail_subscription(&runtime, resource_request);
    let subscription_evidence =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("declaration_subscription"),
                admitted
                    .admitted_subscription_identity()
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("declaration_strategy"),
                admitted
                    .signal_strategy()
                    .strategy_identity()
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("activation_subscription"),
                admitted
                    .admitted_subscription_identity()
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("activation_strategy"),
                admitted
                    .signal_strategy()
                    .strategy_identity()
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting(),
            )
            .seal();
    let declaration = basis_lifecycle()
        .runtime_snapshot("phase-six-subscription", "runtime-bridge-subscription")
        .declare_subscription()
        .expect("subscription declaration basis should admit");
    let activation = basis_lifecycle()
        .runtime_snapshot("phase-six-subscription", "runtime-bridge-subscription")
        .activate_subscription()
        .expect("subscription activation basis should admit");
    for scoped_digest in [
        readmit_lower_runtime_evidence(
            declaration,
            LowerRuntimeBasisEvidence::from_runtime_bridge_facade(
                "runtime-bridge-subscription",
                subscription_evidence.as_str(),
                1,
            ),
        )
        .expect("subscription declaration should use canonical scoped readmission")
        .lower_runtime_binding_digest(),
        readmit_lower_runtime_evidence(
            activation,
            LowerRuntimeBasisEvidence::from_runtime_bridge_facade(
                "runtime-bridge-subscription",
                subscription_evidence.as_str(),
                1,
            ),
        )
        .expect("subscription activation should use canonical scoped readmission")
        .lower_runtime_binding_digest(),
    ] {
        assert!(!scoped_digest.is_empty());
    }

    readmission_row(
        WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromSubscriptionEvidence,
        "Basis readmission from subscription evidence",
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "basis-subscription-readmission-subject",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("subscription"),
            &subscription_evidence,
        )
        .seal(),
        subscription_evidence.clone(),
        subscription_evidence,
    )
}

fn readmission_row(
    seam_key: WorthQueryLowerRuntimeSeamKey,
    capability_label: &str,
    subject_identity: crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity,
    eligibility_evidence: WorthQueryEvidenceIdentity,
    retained_evidence: crate::evidence_identity::WorthQueryEvidenceIdentity,
) -> RepresentativeArtifacts {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        seam_key,
        WorthQueryLowerRuntimeRouteKind::ReadmissionHandoff,
        WorthQueryLowerRuntimeAuthorityOwner::RuntimeBridge,
        capability_label,
        subject_identity,
    );
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request.clone(),
        &eligibility_evidence,
    );
    let retained_evidence_identity =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "phase-six-readmission",
            &retained_evidence,
        );
    let readmission = WorthQueryLowerRuntimeReadmissionReceipt::new(
        eligibility.clone(),
        &retained_evidence_identity,
    );
    let boundary_receipt =
        WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_readmission_receipt(&readmission);
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_readmission_receipt(
        seam_key,
        &readmission,
        &boundary_receipt,
    );
    RepresentativeArtifacts {
        seam_key,
        request,
        eligibility,
        route_plan: None,
        boundary_receipt,
        envelope,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    }
}
