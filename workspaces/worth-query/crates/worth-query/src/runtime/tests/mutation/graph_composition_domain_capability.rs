use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    materialize_graph_composition_domain_invariant_denial,
    prepare_admitted_domain_capability_contribution_for_materialization,
    WorthQueryInvariantCapabilityContributionAuthoring,
};
use worth_proof::TransitionOutcome;

#[test]
fn contributed_domain_invariant_denial_preserves_owner_evidence() {
    let denial = success(materialize_graph_composition_domain_invariant_denial(
        ready_lower_runtime_invariant(
            WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(
                "non_manifold_topology",
                ["HalfEdge", "HalfEdgeNextRelation"],
                ["draft-half-edge"],
                ["mixed_existing_and_symbolic_entity_identity_edges"],
                ["mixed_existing_target_verified_retarget"],
                "program-digest",
                "breadth-digest",
                "components=2;symbolic_entities=1;symbolic_relations=0;declared_collections=2;declared_symbols=1;target_combinations=1;lifecycle_families=1",
                "topology.non_manifold_edge_split",
                "loop successor rewire would create a non-manifold adjacency fanout",
            ).expect("fixture digest must fit"),
        ),
    ).expect("fixture digest must fit"));
    let expected_digest = denial.denial_digest().to_string();
    assert_eq!(denial.invariant_family(), "non_manifold_topology");
    assert_eq!(denial.owner_family(), "domain_capability_invariant_owner");
    assert_eq!(denial.denial_digest(), expected_digest);
    assert_eq!(
        denial.domain_invariant_summary().declared_collections(),
        &["HalfEdge".to_string(), "HalfEdgeNextRelation".to_string()]
    );
    assert_eq!(
        denial
            .domain_invariant_summary()
            .target_combination_families(),
        &["mixed_existing_and_symbolic_entity_identity_edges".to_string()]
    );
}

fn ready_lower_runtime_invariant(
    authoring: WorthQueryInvariantCapabilityContributionAuthoring,
) -> crate::domain_capabilities::WorthQueryMaterializationReadyInvariantCapabilityContribution<
    crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
> {
    let target = lower_runtime_target("boundary-graph");
    let requested = authoring
        .bind_to_lower_runtime_boundary_target(target.clone())
        .expect("fixture digest must fit");
    let eligible = success(evaluate_requested_domain_capability_contribution(requested));
    let admitted = success(admit_eligible_domain_capability_contribution(eligible));
    success(
        prepare_admitted_domain_capability_contribution_for_materialization(admitted, target)
            .expect("fixture digest must fit"),
    )
}

fn lower_runtime_target(
    label: &str,
) -> crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget {
    crate::domain_capabilities::WorthQueryLowerRuntimeBoundaryBoundContributionTarget::for_lower_runtime_boundary_envelope(
        &lower_runtime_envelope(label),
    )
}

fn lower_runtime_envelope(
    label: &str,
) -> crate::lower_runtime_routing::WorthQueryLowerRuntimeBoundaryEnvelope {
    let subject_identity =
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "graph-composition-domain-capability-fixture-subject",
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("fixture"),
            label,
        )
        .seal()
        .expect("fixture digest must fit");
    let request = crate::lower_runtime_routing::WorthQueryLowerRuntimeCapabilityRequest::new(
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeAuthorityOwner::Query,
        "signal-invalidation-routing",
        subject_identity,
    )
    .expect("fixture digest must fit");
    let detail_identity = crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
        crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
    )
    .field_value(
        crate::evidence_identity::WorthQueryEvidenceTag::new("fixture_detail"),
        label,
    )
    .seal()
    .expect("fixture digest must fit");
    let eligibility =
        crate::lower_runtime_routing::WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
            request,
            &detail_identity,
        ).expect("fixture digest must fit");
    let route = crate::lower_runtime_routing::WorthQueryLowerRuntimeRoutePlan::new(
        eligibility,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "graph-composition-domain-capability-fixture-route",
            &detail_identity,
        ).expect("fixture digest must fit"),
    );
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "graph-composition-domain-capability-fixture",
            &crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(
                crate::evidence_identity::WorthQueryEvidenceTag::new("fixture_retained"),
                label,
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

    crate::lower_runtime_routing::WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        &route,
        &boundary,
        &retained_evidence,
    )
}

fn success<T, D, S, R, F, O>(outcome: TransitionOutcome<T, D, S, R, F, O>) -> T
where
    D: std::fmt::Debug,
    S: std::fmt::Debug,
    R: std::fmt::Debug,
    F: std::fmt::Debug,
{
    match outcome {
        TransitionOutcome::Success(value) => value,
        TransitionOutcome::Denied(denial) => panic!("expected success, got denial: {denial:?}"),
        TransitionOutcome::Stale(stale) => panic!("expected success, got stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => {
            panic!("expected success, got rebind-required: {rebind:?}")
        }
        _ => panic!("expected success, got non-success transition outcome"),
    }
}
