use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;
use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatus;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityIdentity;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityStatusField;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub fn revoke_current_capability(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
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
    let field = CapabilityStatusField::reference();
    let locator = installed_field(world, field.entity(), field.aspect(), field.field());
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        crate::domain_computation::primary_graph::tests::fixture::CapabilityStatusBinding::encode(
            &CapabilityStatus::Revoked,
        )
        .unwrap(),
    )]));
    crate::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("revoke-live-capability").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: grant.entity_id_for_test(),
                fields,
            }),
        )),
    );
}

pub fn installed_field(
    world: &AuthorizationWorld,
    entity: &str,
    aspect: &str,
    field: &str,
) -> AspectFieldLocator {
    (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .layout_for_test()
        .field_locator_for_test(entity, aspect, field)
        .unwrap()
        .clone()
}
