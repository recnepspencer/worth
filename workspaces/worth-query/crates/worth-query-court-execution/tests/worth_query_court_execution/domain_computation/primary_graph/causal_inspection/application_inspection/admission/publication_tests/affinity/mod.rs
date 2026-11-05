//! Public and governed binding paths retain exact session target and subject.
use crate::domain_computation::primary_graph::causal_inspection::application_inspection::admission::publication_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::authorization::application_disclosure::decision::binding::WorthQueryApplicationGovernanceBinding;
use worth_relational::facade::identity::EntityId;
pub fn assert_subject_affinity<Schema, Scope>(
    read: &InspectionRead<'_, '_, Schema, Scope>,
    execution: &ExecutionRequest<'_, '_>,
    other_subject: EntityId,
) {
    let target = &(*read.target_for_test());
    let subject = (*read.scope_for_test()).entity_id_for_test();
    assert_ne!(
        subject, other_subject,
        "the issued principal and query scope are genuine distinct records"
    );
    let public = WorthQueryApplicationQueryGovernance::Public;
    let matches = |target: &Arc<WorthQueryApplicationInspectionTarget>, subject| {
        public.inspection_matches_for_test(
            &(*read.session_for_test()),
            target,
            &target.parameters_for_test(),
            (*read.principal_for_test()),
            subject,
        )
    };
    let binding = |target: &Arc<WorthQueryApplicationInspectionTarget>, subject| {
        WorthQueryApplicationGovernanceBinding::from_inspection_session_for_test(
            &(*read.session_for_test()),
            target,
            target.parameters_for_test(),
            (*read.principal_for_test()),
            subject,
        )
    };
    assert!(matches(target, subject));
    assert!(binding(target, subject).is_some());
    assert!(!matches(target, other_subject));
    assert!(binding(target, other_subject).is_none());
    let other_target = WorthQueryApplicationInspectionTarget::capture_for_test(
        execution,
        target.original_for_test().unwrap(),
        None,
        target.contract_for_test().clone(),
        &mut None,
    )
    .unwrap_or_else(|_| panic!("the same original can be retained by a separate genuine target"));
    assert_eq!(other_target.original_subject_for_test(), subject);
    assert!(
        !matches(&other_target, subject),
        "matching source identity is not membership in this session"
    );
    assert!(binding(&other_target, subject).is_none());
}
