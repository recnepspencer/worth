use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationApprover;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationStatusBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationStatusField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewStatusBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewStatusField;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewer;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub type World = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::World;

pub fn elevation_status(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
) -> CapabilityElevationStatus {
    let identity = world
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
            CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
            "elevation-2".into(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = CapabilityElevationStatusField::at("CapabilityElevationStatusField");
    let locator = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .layout_for_test()
        .field_locator_for_test(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let value = observed_field(
        execution,
        &world.application,
        identity.entity_id_for_test(),
        identity.entity_kind_for_test(),
        &locator,
    );
    CapabilityElevationStatusBinding::decode(&value).unwrap()
}

pub fn review_status(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
) -> CapabilityReviewStatus {
    let (review, kind) = resolved_review(phase_custody, execution, world);
    let field = CapabilityReviewStatusField::at("CapabilityReviewStatusField");
    let locator = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .layout_for_test()
        .field_locator_for_test(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let value = observed_field(execution, &world.application, review, kind, &locator);
    CapabilityReviewStatusBinding::decode(&value).unwrap()
}

pub fn has_exact_reviewer(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
    reviewer: EntityId,
) -> bool {
    let (review, _) = resolved_review(phase_custody, execution, world);
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let relation_kind = (*graph
        .layout_for_test()
        .relation(CapabilityReviewer::reference().name())
        .unwrap()
        .kind_for_test());
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    graph
        .integration_handle_for_test()
        .with_runtime_mut_for_test(execution, |runtime| {
            let snapshot = selected.application_basis_for_test().snapshot_handle();
            runtime
                .read_truth()
                .bounded_incoming_relations_of_kind_at_version(
                    execution,
                    review,
                    relation_kind,
                    snapshot.version_id(),
                    4,
                )
                .expect("fixture read request remains live")
                .ok()
                .is_some_and(|read| {
                    let relations = read.into_records();
                    relations.len() == 1
                        && relations[0].source == reviewer
                        && relations[0].target == review
                })
        })
}

pub fn has_exact_approver(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
    approver: EntityId,
) -> bool {
    let elevation = world
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
            CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
            "elevation-2".into(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let relation_kind = (*graph
        .layout_for_test()
        .relation(CapabilityElevationApprover::reference().name())
        .unwrap()
        .kind_for_test());
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    graph
        .integration_handle_for_test()
        .with_runtime_mut_for_test(execution, |runtime| {
            let snapshot = selected.application_basis_for_test().snapshot_handle();
            runtime
                .read_truth()
                .bounded_incoming_relations_of_kind_at_version(
                    execution,
                    elevation.entity_id_for_test(),
                    relation_kind,
                    snapshot.version_id(),
                    4,
                )
                .expect("fixture read request remains live")
                .ok()
                .is_some_and(|read| {
                    let relations = read.into_records();
                    relations.len() == 1
                        && relations[0].source == approver
                        && relations[0].target == elevation.entity_id_for_test()
                })
        })
}

pub fn resolved_review(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
) -> (EntityId, KindId) {
    let identity = world
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
            CapabilityReviewIdentity::at("CapabilityReviewIdentity"),
            "review-2".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    (
        identity.entity_id_for_test(),
        identity.entity_kind_for_test(),
    )
}

pub fn observed_field<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    entity: EntityId,
    kind: KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> worth_foundational::facade::AspectValue
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let graph = (*runtime.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let selected = runtime
        .select_product_branch_for_test(
            execution,
            runtime.product_runtime().default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted");
    graph
        .integration_handle_for_test()
        .with_runtime_mut_for_test(execution, |relational| {
            let snapshot = selected.application_basis_for_test().snapshot_handle();
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::observation::observe_field_value(
                relational, execution, snapshot, entity, kind, locator,
            )
            .expect("native field request remains live")
            .unwrap()
        })
}
