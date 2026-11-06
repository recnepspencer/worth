//! A distinct real governing grant keeps authorization roles separate from the target record.
#[path = "grant/records.rs"]
mod storage_model;
use super::*;
pub use storage_model::*;

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
}
fn key<E>(
    value: &str,
) -> worth_query_host::facade::primary_graph::WorthQueryApplicationEntityKey<
    DocumentRetentionSchema,
    E,
> {
    worth_query_host::facade::primary_graph::WorthQueryApplicationEntityKey::new(value).unwrap()
}
pub fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(ControlGrant::reference(), key(GRANT))
                .field(
                    ControlGrantActionField::at("ControlGrantActionField"),
                    ACTION.to_owned(),
                )
                .field(
                    ControlGrantPurposeField::at("ControlGrantPurposeField"),
                    "reviewed-document".to_owned(),
                )
                .field(
                    ControlGrantStatusField::at("ControlGrantStatusField"),
                    "active".to_owned(),
                )
                .field(
                    ControlGrantWorkflowField::at("ControlGrantWorkflowField"),
                    WorkflowGrantKey::from(SUBJECT),
                )
                .field(
                    ControlGrantNotBeforeField::at("ControlGrantNotBeforeField"),
                    0u64,
                )
                .field(
                    ControlGrantNotAfterField::at("ControlGrantNotAfterField"),
                    u64::MAX,
                )
                .field(
                    ControlGrantDelegationLimitField::at("ControlGrantDelegationLimitField"),
                    0u64,
                ),
        )
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            ControlGrantResource::reference(),
            "control-history-resource",
            key::<ControlGrant>(GRANT),
            key::<WorkflowAuthoringGrant>(SUBJECT),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            ControlGrantGrantor::reference(),
            "control-history-grantor",
            key::<Principal>("document-retention-operator"),
            key::<ControlGrant>(GRANT),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            ControlGrantGrantee::reference(),
            "control-history-grantee",
            key::<Principal>("document-retention-operator"),
            key::<ControlGrant>(GRANT),
        ))
        .unwrap();
}
