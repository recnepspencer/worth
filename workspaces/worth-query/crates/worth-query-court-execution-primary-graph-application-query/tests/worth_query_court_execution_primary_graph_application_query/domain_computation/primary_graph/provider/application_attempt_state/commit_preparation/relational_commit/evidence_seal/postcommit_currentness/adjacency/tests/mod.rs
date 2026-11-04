use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection as DecisionDirection;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
    use worth_relational::facade::runtime::RelationalAdjacencyDirection as NativeDirection;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::adjacency::rebase_decision_adjacency;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
    use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn pair_rebase_uses_native_revision_and_never_retains_partial_facts() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let graph = world.application.runtime.primary_graph().unwrap();
    let relation_kind = graph
        .layout
        .relation(AccountOwner::reference().name())
        .unwrap()
        .kind_for_test());
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    PrincipalIdentityField::reference(),
                    1,
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let fact = Fact::Relation {
        relation_kind,
        from: principal.entity_id_for_test(),
        to: principal.entity_id_for_test(),
        matching_relations: Vec::new(),
    };
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); graph.integration_handle_for_test().with_runtime(request_execution, |runtime| {
            let snapshot = selected.application_basis_for_test().snapshot_handle();

            // A pair's native revision is one selected owner-index lookup.
            assert!(matches!(
                rebase_decision_adjacency(&host_custody.borrowed_request_for_test(), runtime, snapshot, fact.clone()),
                Fact::SourceAdjacencyRevision {
                    comparison_work_limit: 1,
                    native_revision: Some(_),
                    ..
                }
            ));
            assert!(
                matches!(
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::rebase(&host_custody.borrowed_request_for_test(), runtime, snapshot, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase::admit_for_test(vec![fact], [].into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), &std::collections::BTreeSet::new(), true),
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::RebasedSourceFacts::Exact(_)
                ),
                "a complete pair fact rebases through its native structural revision"
            );
            // The admitted work is the decision's recorded limit; a stale
            // anchor or a zero limit acquires no revision.
            let mut stale_anchor = principal.entity_id_for_test();
            stale_anchor.generation.0 += 1;
            let unrevisable = [
                Fact::Relation {
                    relation_kind,
                    from: stale_anchor,
                    to: principal.entity_id_for_test(),
                    matching_relations: Vec::new(),
                },
                Fact::Adjacency {
                    relation_kind,
                    anchor: principal.entity_id_for_test(),
                    direction: DecisionDirection::Outgoing,
                    maximum_work_units: 0,
                    relations: Vec::new(),
                },
            ];
            for fact in unrevisable {
                assert!(
                    matches!(
                        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::rebase(&host_custody.borrowed_request_for_test(), runtime, snapshot, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase::admit_for_test(vec![fact], [].into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), &std::collections::BTreeSet::new(), true),
                        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::RebasedSourceFacts::VerificationRequired { .. }
                    ),
                    "failed relation revision acquisition marks the entire output non-reusable"
                );
            }
            let stale_revision = Fact::SourceAdjacencyRevision {
                relation_kind,
                anchor: principal.entity_id_for_test(),
                direction: NativeDirection::Outgoing,
                native_revision: None,
                comparison_work_limit: 0,
                endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
            };

            assert!(matches!(
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::rebase(&host_custody.borrowed_request_for_test(), runtime, snapshot, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase::admit_for_test(vec![stale_revision.clone()], [].into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None))
                        .unwrap(), &std::collections::BTreeSet::new(), true),
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::RebasedSourceFacts::VerificationRequired { .. }
            ));

            match worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::rebase(&host_custody.borrowed_request_for_test(), runtime, snapshot, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase::admit_for_test(vec![stale_revision.clone()], [].into(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None))
                    .unwrap(), &std::collections::BTreeSet::new(), false) {
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::RebasedSourceFacts::Exact(facts) => {
                    assert_eq!(facts.iter_for_test().cloned().collect::<Vec<_>>(), &[stale_revision])
                }
                other => {
                    panic!("non-output evidence must retain its comparison posture: {other:?}")
                }
            }
        }) }).expect("the fixture read owns its installed host call");
}
