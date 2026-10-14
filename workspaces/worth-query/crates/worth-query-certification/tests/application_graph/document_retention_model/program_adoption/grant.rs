//! Native bootstrap issuance of the adoption grant and its real graph edges.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
pub fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                WorkflowAuthoringGrant::reference(),
                entity_key("program-adoption-grant"),
            )
            .field(
                WorkflowGrantIdentityField::reference(),
                "program-adoption-grant".to_owned(),
            )
            .field(
                WorkflowGrantActionField::reference(),
                "adopt-program".to_owned(),
            )
            .field(
                WorkflowGrantPurposeField::reference(),
                "reviewed-document".to_owned(),
            )
            .field(WorkflowGrantStatusField::reference(), "active".to_owned())
            .field(
                WorkflowGrantWorkflowField::reference(),
                DOCUMENT_IDENTITY.to_owned(),
            )
            .field(WorkflowGrantNotBeforeField::reference(), 0_u64)
            .field(WorkflowGrantNotAfterField::reference(), u64::MAX)
            .field(WorkflowGrantDelegationLimitField::reference(), 0_u64),
        )
        .expect("workflow authoring grant must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantResource::reference(),
            "program-adoption-grant-resource",
            entity_key::<WorkflowAuthoringGrant>("program-adoption-grant"),
            entity_key::<Document>("document-row-1"),
        ))
        .expect("workflow grant resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantRelated::reference(),
            "program-adoption-grant-related",
            entity_key::<WorkflowAuthoringGrant>("program-adoption-grant"),
            entity_key::<Document>("document-row-2"),
        ))
        .expect("workflow grant related subject must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "program-adoption-grant-grantor",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("program-adoption-grant"),
        ))
        .expect("workflow grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "program-adoption-grant-grantee",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("program-adoption-grant"),
        ))
        .expect("workflow grantee must seed");
}
fn entity_key<Entity>(
    identity: &str,
) -> WorthQueryApplicationEntityKey<DocumentRetentionSchema, Entity> {
    WorthQueryApplicationEntityKey::new(identity).unwrap()
}
