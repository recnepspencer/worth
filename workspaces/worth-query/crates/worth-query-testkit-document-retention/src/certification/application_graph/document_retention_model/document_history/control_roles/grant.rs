//! A distinct real governing grant keeps authorization roles separate from the target record.
use super::*;
worth_query_entity!(pub ControlGrant for DocumentRetentionSchema);
worth_query_aspect!(pub ControlGrantFacts for DocumentRetentionSchema, ControlGrant; identity = AspectIdentity(0x91820320), revision = AspectContractRevision(1),);
worth_query_field!(pub ControlGrantActionField for DocumentRetentionSchema, ControlGrant, ControlGrantFacts: String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub ControlGrantPurposeField for DocumentRetentionSchema, ControlGrant, ControlGrantFacts: String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub ControlGrantStatusField for DocumentRetentionSchema, ControlGrant, ControlGrantFacts: String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ControlGrantWorkflowField for DocumentRetentionSchema, ControlGrant, ControlGrantFacts: String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ControlGrantNotBeforeField for DocumentRetentionSchema, ControlGrant, ControlGrantFacts: u64 => U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ControlGrantNotAfterField for DocumentRetentionSchema, ControlGrant, ControlGrantFacts: u64 => U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ControlGrantDelegationLimitField for DocumentRetentionSchema, ControlGrant, ControlGrantFacts: u64 => U64ApplicationValueBinding, read_write, no_equality);
worth_query_relation!(pub ControlGrantResource in DocumentRetentionSchema, ControlGrant => WorkflowAuthoringGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ControlGrantParent in DocumentRetentionSchema, ControlGrant => ControlGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ControlGrantGrantor in DocumentRetentionSchema, Principal => ControlGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ControlGrantGrantee in DocumentRetentionSchema, Principal => ControlGrant; integrity = same_context_unbounded_retain_dangling);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .entity(ControlGrant::reference())
        .aspect(ControlGrant::reference(), ControlGrantFacts::reference())
        .field(
            ControlGrant::reference(),
            ControlGrantActionField::reference(),
        )
        .field(
            ControlGrant::reference(),
            ControlGrantPurposeField::reference(),
        )
        .field(
            ControlGrant::reference(),
            ControlGrantStatusField::reference(),
        )
        .field(
            ControlGrant::reference(),
            ControlGrantWorkflowField::reference(),
        )
        .field(
            ControlGrant::reference(),
            ControlGrantNotBeforeField::reference(),
        )
        .field(
            ControlGrant::reference(),
            ControlGrantNotAfterField::reference(),
        )
        .field(
            ControlGrant::reference(),
            ControlGrantDelegationLimitField::reference(),
        )
        .relation(
            ControlGrantResource::reference(),
            ControlGrant::reference(),
            WorkflowAuthoringGrant::reference(),
        )
        .relation(
            ControlGrantParent::reference(),
            ControlGrant::reference(),
            ControlGrant::reference(),
        )
        .relation(
            ControlGrantGrantor::reference(),
            Principal::reference(),
            ControlGrant::reference(),
        )
        .relation(
            ControlGrantGrantee::reference(),
            Principal::reference(),
            ControlGrant::reference(),
        )
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
                .field(ControlGrantActionField::reference(), ACTION.to_owned())
                .field(
                    ControlGrantPurposeField::reference(),
                    "reviewed-document".to_owned(),
                )
                .field(ControlGrantStatusField::reference(), "active".to_owned())
                .field(ControlGrantWorkflowField::reference(), SUBJECT.to_owned())
                .field(ControlGrantNotBeforeField::reference(), 0u64)
                .field(ControlGrantNotAfterField::reference(), u64::MAX)
                .field(ControlGrantDelegationLimitField::reference(), 0u64),
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
