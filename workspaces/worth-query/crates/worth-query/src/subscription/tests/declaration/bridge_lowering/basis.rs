use super::*;
use crate::live::LiveQueryFamily;

#[test]
fn basis_request_digest_changes_by_basis_posture() {
    let declaration = |basis, policy: &str, tenant: &str| {
        let input = LiveQueryAdmissionArtifact::for_test_with_context(
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionConstructionSource::FacadeLive,
            basis,
            QuerySubscriptionFutureSelection::ordinary().expect("fixture digest must fit"),
            Some(policy.to_string()),
            Some(tenant.to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )
        .expect("fixture digest must fit");
        let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
        declare_query_subscription(selection, roomy_slice_budget()).unwrap()
    };

    let current = lower_query_subscription_to_bridge(
        declaration(
            QuerySubscriptionBasisPosture::CurrentHead,
            "policy",
            "tenant",
        ),
        roomy_lowering_budget(),
    )
    .unwrap();
    let branch = lower_query_subscription_to_bridge(
        declaration(
            QuerySubscriptionBasisPosture::BranchHead,
            "policy",
            "tenant",
        ),
        roomy_lowering_budget(),
    )
    .unwrap();
    let snapshot = lower_query_subscription_to_bridge(
        declaration(
            QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot,
            "policy",
            "tenant",
        ),
        roomy_lowering_budget(),
    )
    .unwrap();
    let changed_policy = lower_query_subscription_to_bridge(
        declaration(
            QuerySubscriptionBasisPosture::CurrentHead,
            "policy-v2",
            "tenant",
        ),
        roomy_lowering_budget(),
    )
    .unwrap();
    let changed_tenant = lower_query_subscription_to_bridge(
        declaration(
            QuerySubscriptionBasisPosture::CurrentHead,
            "policy",
            "tenant-beta",
        ),
        roomy_lowering_budget(),
    )
    .unwrap();

    assert_ne!(
        current.basis_request().basis_binding_projection().label(),
        branch.basis_request().basis_binding_projection().label()
    );
    assert_ne!(
        current.basis_request().basis_binding_projection().label(),
        snapshot.basis_request().basis_binding_projection().label()
    );
    assert_ne!(
        current.basis_request().basis_binding_projection().label(),
        changed_policy
            .basis_request()
            .basis_binding_projection()
            .label()
    );
    assert_ne!(
        current.basis_request().basis_binding_projection().label(),
        changed_tenant
            .basis_request()
            .basis_binding_projection()
            .label()
    );
    assert_eq!(
        current
            .basis_request()
            .source_declaration_projection()
            .label(),
        current.query_declaration_projection().label()
    );
    assert_eq!(
        snapshot.basis_request().request_kind(),
        &QuerySubscriptionBasisBindingRequestKind::RuntimeSnapshot
    );
}

#[test]
fn basis_request_tracks_declaration_owned_future_identity_without_local_reclassification() {
    let declaration = |future_selection| {
        let input = LiveQueryAdmissionArtifact::for_test_with_context(
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionConstructionSource::FacadeLive,
            QuerySubscriptionBasisPosture::CurrentHead,
            future_selection,
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )
        .expect("fixture digest must fit");
        let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
        declare_query_subscription(selection, roomy_slice_budget()).unwrap()
    };

    let ordinary = lower_query_subscription_to_bridge(
        declaration(QuerySubscriptionFutureSelection::ordinary().expect("fixture digest must fit")),
        roomy_lowering_budget(),
    )
    .unwrap();
    let async_edge_42 = lower_query_subscription_to_bridge(
        declaration(
            QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![QuerySubscriptionAsyncRequestIdentityPart::new(
                    "edge", "edge:42",
                )],
            )
            .expect("fixture digest must fit"),
        ),
        roomy_lowering_budget(),
    )
    .unwrap();
    let async_edge_77 = lower_query_subscription_to_bridge(
        declaration(
            QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![QuerySubscriptionAsyncRequestIdentityPart::new(
                    "edge", "edge:77",
                )],
            )
            .expect("fixture digest must fit"),
        ),
        roomy_lowering_budget(),
    )
    .unwrap();

    assert_ne!(
        ordinary.query_declaration_projection().label(),
        async_edge_42.query_declaration_projection().label()
    );
    assert_ne!(
        async_edge_42.query_declaration_projection().label(),
        async_edge_77.query_declaration_projection().label()
    );
    assert_ne!(
        ordinary.basis_request().basis_binding_projection().label(),
        async_edge_42
            .basis_request()
            .basis_binding_projection()
            .label()
    );
    assert_ne!(
        async_edge_42
            .basis_request()
            .basis_binding_projection()
            .label(),
        async_edge_77
            .basis_request()
            .basis_binding_projection()
            .label()
    );
}
