use super::*;
use crate::basis_lifecycle::basis_lifecycle;
use crate::live::LiveQueryFamily;

#[test]
fn construction_source_does_not_change_exact_reuse_subscription_meaning() {
    let mut selections = [
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
        select_query_subscription_family(input, roomy_budget()).unwrap()
    });
    let first = selections.next().unwrap();

    for selection in selections {
        assert_eq!(first.family(), selection.family());
        assert_eq!(
            first.equivalence_basis().equivalence_projection().label(),
            selection
                .equivalence_basis()
                .equivalence_projection()
                .label()
        );
    }
}

#[test]
fn meaning_digest_changes_for_policy_tenant_or_relationship_proof_context() {
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

    let baseline = select_query_subscription_family(baseline, roomy_budget()).unwrap();
    let changed_policy = select_query_subscription_family(changed_policy, roomy_budget()).unwrap();
    let changed_tenant = select_query_subscription_family(changed_tenant, roomy_budget()).unwrap();
    let changed_proof = select_query_subscription_family(changed_proof, roomy_budget()).unwrap();

    assert_ne!(
        baseline
            .equivalence_basis()
            .equivalence_projection()
            .label(),
        changed_policy
            .equivalence_basis()
            .equivalence_projection()
            .label()
    );
    assert_ne!(
        baseline
            .equivalence_basis()
            .equivalence_projection()
            .label(),
        changed_tenant
            .equivalence_basis()
            .equivalence_projection()
            .label()
    );
    assert_ne!(
        baseline
            .equivalence_basis()
            .equivalence_projection()
            .label(),
        changed_proof
            .equivalence_basis()
            .equivalence_projection()
            .label()
    );
}

#[test]
fn distinct_branch_basis_proofs_never_collapse_to_one_subscription_meaning() {
    let mut branch_alpha = LiveQueryAdmissionArtifact::for_test_with_context(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
        QuerySubscriptionBasisPosture::BranchHead,
        QuerySubscriptionFutureSelection::ordinary().expect("fixture digest must fit"),
        Some("policy".to_string()),
        Some("tenant".to_string()),
        Some("relationship-proof".to_string()),
        QuerySubscriptionRelationshipProofPosture::Admitted,
    )
    .expect("fixture digest must fit");
    let mut branch_beta = branch_alpha.clone();
    branch_alpha.scoped_declaration_basis = Some(
        basis_lifecycle()
            .branch_head("alpha", true)
            .declare_subscription()
            .unwrap(),
    );
    branch_beta.scoped_declaration_basis = Some(
        basis_lifecycle()
            .branch_head("beta", true)
            .declare_subscription()
            .unwrap(),
    );

    let alpha = select_query_subscription_family(branch_alpha, roomy_budget()).unwrap();
    let beta = select_query_subscription_family(branch_beta, roomy_budget()).unwrap();

    assert_eq!(alpha.basis_posture(), beta.basis_posture());
    assert_ne!(
        alpha.equivalence_basis().evidence_identity(),
        beta.equivalence_basis().evidence_identity()
    );
}
