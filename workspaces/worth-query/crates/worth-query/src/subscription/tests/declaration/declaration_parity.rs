use super::*;
use crate::live::LiveQueryFamily;

fn declare(input: LiveQueryAdmissionArtifact) -> QuerySubscriptionDeclarationArtifact {
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    declare_query_subscription(selection, roomy_slice_budget()).unwrap()
}

#[test]
fn equivalent_construction_sources_declare_identical_subscription_digest() {
    let mut declarations = [
        QuerySubscriptionConstructionSource::Direct,
        QuerySubscriptionConstructionSource::ScopeExpanded,
        QuerySubscriptionConstructionSource::TemplateInstantiated,
        QuerySubscriptionConstructionSource::SavedExactReuse,
        QuerySubscriptionConstructionSource::FacadeLive,
    ]
    .into_iter()
    .map(|source| {
        let input = LiveQueryAdmissionArtifact::for_test(LiveQueryFamily::Detail, None, source)
            .expect("fixture digest must fit");
        let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
        declare_query_subscription(selection, roomy_slice_budget()).unwrap()
    });

    let first = declarations.next().unwrap();
    for declaration in declarations {
        assert_eq!(first.family(), declaration.family());
        assert_eq!(
            first.declaration_projection().label(),
            declaration.declaration_projection().label()
        );
        assert_eq!(
            first.equivalence_projection().label(),
            declaration.equivalence_projection().label()
        );
    }
}

#[test]
fn declaration_digest_changes_for_policy_tenant_or_relationship_proof_context() {
    let baseline = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");
    let changed_policy = LiveQueryAdmissionArtifact::for_test_with_context(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
        QuerySubscriptionBasisPosture::CurrentHead,
        QuerySubscriptionFutureSelection::ordinary().expect("fixture digest must fit"),
        Some("policy-v2".to_string()),
        Some("tenant".to_string()),
        Some("relationship-proof".to_string()),
        QuerySubscriptionRelationshipProofPosture::Admitted,
    )
    .expect("fixture digest must fit");
    let changed_tenant = LiveQueryAdmissionArtifact::for_test_with_context(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
        QuerySubscriptionBasisPosture::CurrentHead,
        QuerySubscriptionFutureSelection::ordinary().expect("fixture digest must fit"),
        Some("policy".to_string()),
        Some("tenant-beta".to_string()),
        Some("relationship-proof".to_string()),
        QuerySubscriptionRelationshipProofPosture::Admitted,
    )
    .expect("fixture digest must fit");
    let changed_proof = LiveQueryAdmissionArtifact::for_test_with_context(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
        QuerySubscriptionBasisPosture::CurrentHead,
        QuerySubscriptionFutureSelection::ordinary().expect("fixture digest must fit"),
        Some("policy".to_string()),
        Some("tenant".to_string()),
        Some("relationship-proof-v2".to_string()),
        QuerySubscriptionRelationshipProofPosture::Admitted,
    )
    .expect("fixture digest must fit");

    let baseline = declare(baseline);
    let changed_policy = declare(changed_policy);
    let changed_tenant = declare(changed_tenant);
    let changed_proof = declare(changed_proof);

    assert_ne!(
        baseline.declaration_projection().label(),
        changed_policy.declaration_projection().label()
    );
    assert_ne!(
        baseline.declaration_projection().label(),
        changed_tenant.declaration_projection().label()
    );
    assert_ne!(
        baseline.declaration_projection().label(),
        changed_proof.declaration_projection().label()
    );
}

#[test]
fn temporal_basis_and_async_request_identity_change_declaration_meaning_explicitly() {
    let temporal_current = declare(
        LiveQueryAdmissionArtifact::for_test_with_context(
            LiveQueryFamily::OrderedCollection,
            None,
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionBasisPosture::CurrentHead,
            QuerySubscriptionFutureSelection::temporal().expect("fixture digest must fit"),
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )
        .expect("fixture digest must fit"),
    );
    let temporal_branch = declare(
        LiveQueryAdmissionArtifact::for_test_with_context(
            LiveQueryFamily::OrderedCollection,
            None,
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionBasisPosture::BranchHead,
            QuerySubscriptionFutureSelection::temporal().expect("fixture digest must fit"),
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )
        .expect("fixture digest must fit"),
    );
    let async_edge_42 = declare(
        LiveQueryAdmissionArtifact::for_test_with_context(
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionBasisPosture::CurrentHead,
            QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![
                    QuerySubscriptionAsyncRequestIdentityPart::new("edge", "edge:42"),
                    QuerySubscriptionAsyncRequestIdentityPart::new("material", "mat:blue"),
                ],
            )
            .expect("fixture digest must fit"),
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )
        .expect("fixture digest must fit"),
    );
    let async_edge_77 = declare(
        LiveQueryAdmissionArtifact::for_test_with_context(
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionBasisPosture::CurrentHead,
            QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![
                    QuerySubscriptionAsyncRequestIdentityPart::new("edge", "edge:77"),
                    QuerySubscriptionAsyncRequestIdentityPart::new("material", "mat:blue"),
                ],
            )
            .expect("fixture digest must fit"),
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )
        .expect("fixture digest must fit"),
    );
    let async_edge_42_reordered = declare(
        LiveQueryAdmissionArtifact::for_test_with_context(
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionBasisPosture::CurrentHead,
            QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![
                    QuerySubscriptionAsyncRequestIdentityPart::new("material", "mat:blue"),
                    QuerySubscriptionAsyncRequestIdentityPart::new("edge", "edge:42"),
                    QuerySubscriptionAsyncRequestIdentityPart::new("edge", "edge:42"),
                ],
            )
            .expect("fixture digest must fit"),
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )
        .expect("fixture digest must fit"),
    );

    assert_ne!(
        temporal_current.declaration_projection().label(),
        temporal_branch.declaration_projection().label()
    );
    assert_ne!(
        async_edge_42.declaration_projection().label(),
        async_edge_77.declaration_projection().label()
    );
    assert_eq!(
        async_edge_42.declaration_projection().label(),
        async_edge_42_reordered.declaration_projection().label()
    );
    assert_eq!(
        async_edge_42
            .future_selection()
            .async_request_identity()
            .len(),
        2
    );
}
