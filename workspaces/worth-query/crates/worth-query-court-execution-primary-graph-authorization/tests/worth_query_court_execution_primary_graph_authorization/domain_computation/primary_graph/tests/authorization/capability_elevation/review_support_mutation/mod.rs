use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;
use std::collections::BTreeMap;

use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::RelationSpec;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityCustodian;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityGrantor;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewStatusField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewer;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub fn complete_review_out_of_band(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    reviewer: EntityId,
) {
    let review = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application_for_test(&world.application),
            CapabilityReviewIdentity::reference(),
            "review-2".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = CapabilityReviewStatusField::reference();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let locator = graph
        .layout_for_test()
        .field_locator_for_test(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let relation_kind = (*graph
        .layout_for_test()
        .relation(CapabilityReviewer::reference().name())
        .unwrap()
        .kind_for_test());
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewStatusBinding::encode(
            &CapabilityReviewStatus::Completed,
        )
        .unwrap(),
    )]));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("complete-review-out-of-band")
            .push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: review.entity_id_for_test(),
                    fields,
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: relation_kind,
                    client_key: ClientKey::raw("out-of-band-reviewer"),
                    source: EntityReference::Existing(reviewer),
                    target: EntityReference::Existing(review.entity_id_for_test()),
                    fields: AspectFieldPatch::default(),
                },
            ))),
    );
}

pub fn replace_support_grantor_with_custodian(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    principal: EntityId,
) {
    let grant = world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application_for_test(&world.application),
            CapabilityIdentity::reference(),
            "capability-1".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let grantor_kind = (*graph
        .layout_for_test()
        .relation(CapabilityGrantor::reference().name())
        .unwrap()
        .kind_for_test());
    let custodian_kind = (*graph
        .layout_for_test()
        .relation(CapabilityCustodian::reference().name())
        .unwrap()
        .kind_for_test());
    let handle = graph.integration_handle_for_test();
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let grantor = handle.with_runtime_mut_for_test(execution, |runtime| {
        let snapshot = selected.application_basis_for_test().snapshot_handle();

        runtime
            .read_truth()
            .visible_relations_of_kind(execution, grantor_kind, snapshot.version_id())
            .expect("fixture read request remains live")
            .into_iter()
            .find(|record| {
                record.source == principal && record.target == grant.entity_id_for_test()
            })
            .expect("the request support has one current grantor path")
            .relation_id
    });
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("replace-elevation-support-policy-path")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: grantor,
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: custodian_kind,
                    client_key: ClientKey::raw("capability-1-custodian"),
                    source: EntityReference::Existing(principal),
                    target: EntityReference::Existing(grant.entity_id_for_test()),
                    fields: AspectFieldPatch::default(),
                },
            ))),
    );
}
