//! A Current acknowledgment cannot discharge a cause on a different Product.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::required_work::RequiredWorkWorthQueryOutputDemandRegistryForTest;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::current_basis::CurrentBasis;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::PerformedMembers;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::denial;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;
#[test]
fn a_current_proof_from_another_basis_acknowledges_nothing() {
    with_committed_own_write(|world, _receipt, _recorded, _resources| {
        world.application.with_host_advancement(|phase| {
        let execution = phase.execution_request_for_for_test(&(*world.application.product_runtime_for_test())).unwrap();
        let runtime = &world.application;

        let (basis_a, _) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::selection::select_required_basis(
            &execution,
            runtime,
            runtime.current_world(),
        )
        .unwrap();
        let a = CurrentBasis::of(basis_a.selected().product().observation_for_test());
        let registry = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry::default();
        // The row's key names this World's own occurrence. The shared registry
        // fixture key would install a second fixture World inside this
        // advancement, which the advancement owner refuses as a nested opening.
        let (interest, member) = registry.fixture_scope_work_membership(
            basis_a
                .selected()
                .product()
                .observation_for_test()
                .lifecycle_incarnation(),
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        );
        let proof = PerformedMembers::start().current_for_test(
            (),
            a.clone(),
            member.certify_current_basis(a.clone()),
        );
        // Use the registry's real queue, so the writer's state is observable.
        {
            use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
            use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
            use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
            use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;

            use worth_relational::facade::transactions::WorkerIntentBatch;
            use worth_relational::facade::transactions::MutationIntent;
            use worth_relational::facade::transactions::EntityMutationIntent;
            use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
            use worth_relational::facade::transactions::AspectFieldPatch;
            let root = resolved_account(&phase.test_host_request_for_test(), world, "unrelated", &live_scope());
            let field = AccountLabel::reference();
            let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
            let locator = graph.layout_for_test().field_locator_for_test(field.entity(), field.aspect(), field.field()).unwrap().clone();
            publish_relational_mutation(&execution, world,
                WorkerIntentBatch::new("certification-unrelated-source").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: root.entity_id_for_test(),
                        fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                            locator, worth_foundational::facade::AspectValue::String("unrelated-write".into()))])),
                    }))));
        }
        let (basis_b, _) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::selection::select_required_basis(
            &execution,
            runtime,
            runtime.current_world(),
        )
        .unwrap();
        let b = CurrentBasis::of(basis_b.selected().product().observation_for_test());
        assert_ne!(a, b, "the production publication really moved the basis");
        member.certify_current_basis(b.clone());
        let work = member.fixture_selected_work();
        let outcome = work.acknowledge_current_admitted_for_test(proof);
        assert!(
            matches!(outcome, Err(ref denial) if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignDemand),
            "the row itself refuses the old certification basis"
        );

        assert!(
            member.fixture_is_unresolved_for_test(),
            "a proof certified before this publication cannot acknowledge its record"
        );
        let proof_b = PerformedMembers::start().current_for_test(
            (),
            b.clone(),
            member.certify_current_basis(b.clone()),
        );
        let work = member.fixture_selected_work();
        drop(
            work.acknowledge_current_admitted_for_test(proof_b)
                .unwrap(),
        );
        assert!(
            !member.fixture_is_unresolved_for_test(),
            "the new inspection acknowledges its own basis"
        );
        drop(interest);
        }).expect("the owner proof owns its request");
    });
}
#[test]
fn a_current_proof_of_record_a_cannot_acknowledge_record_b_at_the_same_basis() {
    with_committed_own_write(|world, _, _, _| {
        world.application.with_host_advancement(|phase| {
        let execution = phase.execution_request_for_for_test(&(*world.application.product_runtime_for_test())).unwrap();
        let runtime = &world.application;

        let (selected, _) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::selection::select_required_basis(
            &execution,
            runtime,
            runtime.current_world(),
        )
        .unwrap();
        let basis = CurrentBasis::of(selected.selected().product().observation_for_test());
        let registry = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry::default();
        let occurrence = selected
            .selected()
            .product()
            .observation_for_test()
            .lifecycle_incarnation();
        let (interest_a, member_a) = registry.fixture_scope_work_membership(
            occurrence,
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        );
        let (interest_b, member_b) = registry.fixture_scope_work_membership(
            occurrence,
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                2,
                1,
            ),
        );
        assert_ne!(member_a.key_for_test(), member_b.key_for_test());
        let proof = PerformedMembers::start().current_for_test(
            (),
            basis.clone(),
            member_a.certify_current_basis(basis.clone()),
        );
        // Both records are certified at the same basis; only identity differs.
        drop(member_b.certify_current_basis(basis.clone()));
        let _work_a = member_a.fixture_selected_work();
        let work = member_b.fixture_selected_work();
        let result = work.acknowledge_current_admitted_for_test(proof);
        assert!(
            matches!(result, Err(ref denial) if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignDemand),
            "the writer refuses another record's Current proof by value"
        );
        assert!(
            member_b.fixture_is_unresolved_for_test(),
            "record B's initial cause is unchanged"
        );
        assert!(
            member_a.fixture_is_unresolved_for_test(),
            "certification has not acknowledged record A either"
        );
        drop((interest_a, interest_b));
        }).expect("the owner proof owns its request");
    });
}
