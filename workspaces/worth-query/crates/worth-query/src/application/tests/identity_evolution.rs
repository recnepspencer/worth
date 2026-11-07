use super::*;

#[test]
fn support_report_includes_identity_evolution_capability_and_profile() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let support = facade.support_matrix();
    let report = facade.support_report().expect("fixture digest must fit");

    assert_eq!(
        support
            .descriptor(WorthQueryCapabilityFamily::IdentityEvolution)
            .expect("identity evolution descriptor should exist")
            .status(),
        WorthQueryCapabilityStatus::Admitted
    );
    assert!(report
        .admitted_capability_families()
        .contains(&WorthQueryCapabilityFamily::IdentityEvolution));
    let profile = report
        .identity_evolution_support_profile()
        .expect("identity evolution profile should be present");
    assert_eq!(
        profile
            .admitted_traversal_families()
            .iter()
            .map(|family| family.as_str())
            .collect::<Vec<_>>(),
        vec![
            "direct_predecessor",
            "direct_successor",
            "direct_replacement",
            "direct_split_successors",
            "direct_merge_successor",
            "branch_local_direct_evolution"
        ]
    );
}

#[test]
fn identity_evolution_capability_admits_and_executes_query_surface() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let identity_evolution = facade
        .identity_evolution_capability()
        .expect("runtime-backed facade should admit identity evolution");
    let admission = identity_evolution.admission().clone();

    let admitted = identity_evolution
        .capability()
        .admit_query(
            IdentityEvolutionQueryContext::correspondence_identity_comparison_for_test(
                crate::identity::CanonicalQueryDigest::from_parts(&["app:identity".to_string()]),
                IdentityEvolutionComparisonBasisFamily::BranchToBranch,
                crate::identity::BasisDigest::from_parts(&["basis:left".to_string()]),
                crate::identity::BasisDigest::from_parts(&["basis:right".to_string()]),
                CorrespondenceIdentityComparison::advisory_between("left-id", "right-id"),
            ),
        )
        .expect("identity evolution comparison should admit");
    let execution = identity_evolution
        .capability()
        .execute_query(&admitted)
        .expect("identity evolution comparison should execute");

    assert_eq!(
        admission.descriptor().family(),
        WorthQueryCapabilityFamily::IdentityEvolution
    );
    assert_eq!(execution.family().as_str(), "branch_to_branch_comparison");
    assert!(execution
        .result_bundle()
        .as_advisory_identity_candidate_set()
        .is_some());
    assert_eq!(identity_evolution.counters().capability_lookup_count(), 1);
}
