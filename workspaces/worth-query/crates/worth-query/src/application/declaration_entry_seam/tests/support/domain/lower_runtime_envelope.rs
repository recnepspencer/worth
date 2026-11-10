pub fn lower_runtime_envelope(
    target_digest: &str,
) -> crate::runtime::WorthQueryLowerRuntimeBoundaryEnvelope {
    let request = crate::lower_runtime_routing::WorthQueryLowerRuntimeCapabilityRequest::new(
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeAuthorityOwner::Query,
        "signal-invalidation-routing",
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "declaration-entry-seam-target",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("test_target"),
            target_digest,
        )
        .seal()
        .expect("fixture digest must fit"),
    )
    .expect("fixture digest must fit");
    let detail_identity = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("test_detail"),
        "detail",
    )
    .seal()
    .expect("fixture digest must fit");
    let eligibility = crate::lower_runtime_routing::WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request,
        &detail_identity,
    ).expect("fixture digest must fit");
    let route = crate::lower_runtime_routing::WorthQueryLowerRuntimeRoutePlan::new(
        eligibility,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "declaration-entry-seam-route",
            &detail_identity,
        ).expect("fixture digest must fit"),
    );
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "declaration-entry-seam-test",
            &crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(
                crate::evidence_identity::WorthQueryEvidenceTag::new("test_retained"),
                format!("retained:{target_digest}"),
            )
            .seal()
            .expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit");
    let boundary =
        crate::lower_runtime_routing::WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
            &route,
            &retained_evidence,
        );
    crate::runtime::WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        &route,
        &boundary,
        &retained_evidence,
    )
}
