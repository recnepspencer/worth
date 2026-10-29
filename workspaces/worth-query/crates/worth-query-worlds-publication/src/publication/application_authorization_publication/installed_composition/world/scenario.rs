pub use worth_query_declaration::facade::application_schema::ApplicationRelationRef;
pub use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationRelationSeed,
    WorthQueryPrimaryGraphBootstrap,
};

pub use super::super::declaration::*;
pub use super::super::CompositionScenario;

pub fn bind(
    graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>,
    scenario: CompositionScenario,
) {
    if !matches!(scenario, CompositionScenario::MissingAuthorization) {
        bind_resource_relation(graph, ResourceOwner::reference(), "resource-owner");
    }
    if matches!(
        scenario,
        CompositionScenario::ExplicitDeny | CompositionScenario::AccumulatedProhibitions
    ) {
        bind_resource_relation(graph, ExplicitDeny::reference(), "explicit-deny");
    }
    if matches!(scenario, CompositionScenario::AccumulatedProhibitions) {
        bind_accumulated_prohibitions(graph);
    }
}

pub fn bind_accumulated_prohibitions(
    graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>,
) {
    bind_resource_relation(graph, ConflictingActor::reference(), "conflicting-actor");
    bind_action_actor(
        graph,
        RequestActor::reference(),
        "request-actor",
        "selected-request",
    );
    bind_action_actor(
        graph,
        PriorActor::reference(),
        "prior-actor",
        "selected-prior",
    );
}

pub fn bind_resource_relation<Relation>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>,
    relation: ApplicationRelationRef<PublicationAuthorizationSchema, Relation, Principal, Resource>,
    key: &str,
) {
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            relation,
            key,
            entity_key("principal-1"),
            entity_key("resource-1"),
        ))
        .unwrap();
}

pub fn bind_action_actor<Relation>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>,
    relation: ApplicationRelationRef<
        PublicationAuthorizationSchema,
        Relation,
        Principal,
        ActionRecord,
    >,
    key: &str,
    record: &str,
) {
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            relation,
            key,
            entity_key("principal-1"),
            entity_key(record),
        ))
        .unwrap();
}

pub fn entity_key<Entity>(
    value: &str,
) -> WorthQueryApplicationEntityKey<PublicationAuthorizationSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value).unwrap()
}
