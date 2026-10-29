pub use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};

pub use super::super::declaration::*;

pub fn bind(graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>) {
    bind_resource(graph);
    bind_grant(graph);
    bind_action_records(graph);
}

pub fn bind_resource(graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Resource::reference(),
                WorthQueryApplicationEntityKey::new("resource-1").unwrap(),
            )
            .field(ResourceIdentityField::reference(), "resource-1".to_owned())
            .field(ResourceWorkflowField::reference(), "open".to_owned())
            .field(ResourceLabelField::reference(), "protected".to_owned()),
        )
        .unwrap();
}

pub fn bind_grant(graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                CapabilityGrant::reference(),
                WorthQueryApplicationEntityKey::new("grant-1").unwrap(),
            )
            .field(GrantIdentityField::reference(), "grant-1".to_owned())
            .field(GrantActionField::reference(), "inspect".to_owned())
            .field(
                GrantPurposeField::reference(),
                "publication-proof".to_owned(),
            )
            .field(GrantStatusField::reference(), "active".to_owned())
            .field(GrantWorkflowField::reference(), "open".to_owned())
            .field(GrantNotBeforeField::reference(), 0_u64)
            .field(GrantNotAfterField::reference(), u64::MAX)
            .field(GrantDelegationLimitField::reference(), 0_u64),
        )
        .unwrap();
    bind_grant_principal_relations(graph);
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantResource::reference(),
            "grant-resource",
            entity_key("grant-1"),
            entity_key("resource-1"),
        ))
        .unwrap();
}

pub fn bind_grant_principal_relations(
    graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>,
) {
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantee::reference(),
            "grant-grantee",
            entity_key("principal-1"),
            entity_key("grant-1"),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantor::reference(),
            "grant-grantor",
            entity_key("principal-1"),
            entity_key("grant-1"),
        ))
        .unwrap();
}

pub fn bind_action_records(
    graph: &mut WorthQueryPrimaryGraphBootstrap<PublicationAuthorizationSchema>,
) {
    for record in ["selected-request", "selected-prior"] {
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(ActionRecord::reference(), entity_key(record))
                    .field(ActionRecordIdentityField::reference(), record.to_owned()),
            )
            .unwrap();
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                ActionResource::reference(),
                format!("{record}-resource"),
                entity_key(record),
                entity_key("resource-1"),
            ))
            .unwrap();
    }
}

pub fn entity_key<Entity>(
    value: &str,
) -> WorthQueryApplicationEntityKey<PublicationAuthorizationSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value).unwrap()
}
