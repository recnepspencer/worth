//! Ordinary admitted query and owner-published field update, without a second schema.
use crate::domain_computation::primary_graph::application_query::projection::retained_projection::tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotResult;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
pub type Query =
    <TestAccountSourceBinding as ApplicationQueryBinding<IdentityExecutionSchema>>::Query;

pub fn read(
    world: &AuthorizationWorld,
    phase: &WorthQueryAdvancementPhase<'_>,
    status: &str,
    retain: bool,
    bound: bool,
) -> Result<
    WorthQueryApplicationOneShotResult<Query, AccountSummaryResult>,
    WorthQueryApplicationOneShotDenial,
> {
    let request = live_scope();
    let external = world.authenticate("alice", std::time::Duration::from_secs(60), &request);
    let schema = world.application.installed_schema();
    let query = if bound {
        schema
            .installed_query_binding::<TestAccountSourceBinding>()
            .unwrap()
            .into_query()
    } else {
        schema.certification_query(Query::reference()).unwrap()
    };
    let selected = world.selected_product(&phase.test_host_request_for_test());
    let principal = selected
        .resolve_authenticated_principal(
            phase,
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            phase,
            AccountStatus::at("AccountStatus"),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = selected
        .admit_application_query(
            phase,
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), status.to_owned())
                .unwrap(),
            WorthQueryProductQueryControls::new(std::num::NonZeroUsize::MIN, &request),
        )
        .unwrap();
    let plan = if retain {
        plan.retain_projection_for_inspection()
    } else {
        plan
    };
    world
        .application
        .execute_application_query_one_shot(phase, plan)
}

pub fn set_label(
    world: &AuthorizationWorld,
    phase: &WorthQueryAdvancementPhase<'_>,
    entity: worth_relational::facade::identity::EntityId,
    field: &worth_query_installation::facade::WorthQueryInstalledGraphProjection,
    value: &str,
) {
    use worth_foundational::facade::AspectFieldLocator;
    use worth_foundational::facade::CanonicalFieldPath;
    use worth_foundational::facade::LocatorAuthority;
    use worth_relational::facade::transactions::AspectFieldPatch;
    use worth_relational::facade::transactions::EntityMutationIntent;
    use worth_relational::facade::transactions::MutationIntent;
    use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
    use worth_relational::facade::transactions::WorkerIntentBatch;
    let fields = AspectFieldPatch::from(std::collections::BTreeMap::from([(
        AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            field.aspect_key().clone(),
            CanonicalFieldPath::single(field.field_key().clone()),
        ),
        worth_foundational::facade::AspectValue::String(value.into()),
    )]));
    let custody = phase.test_host_request_for_test();
    publish_relational_mutation(
        &custody.borrowed_request_for_test(),
        world,
        WorkerIntentBatch::new("original-projection-label").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields,
            }),
        )),
    );
}
