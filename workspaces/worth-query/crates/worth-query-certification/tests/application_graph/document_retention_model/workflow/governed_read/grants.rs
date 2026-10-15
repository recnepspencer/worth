//! Independently issued read grants use the actual existing native capability contract.
use super::super::super::retention_entry::DOCUMENT_IDENTITY;
use super::super::declaration::*;
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
pub(in super::super::super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>,
) {
    for (grant, action, grantees) in [
        (
            "workflow-subject-read-grant",
            SubjectReadCapability::ACTION,
            &["document-retention-operator", "document-retention-reviewer"][..],
        ),
        (
            "workflow-definition-read-grant",
            DefinitionReadCapability::ACTION,
            &["document-retention-operator"][..],
        ),
        (
            "workflow-audit-read-grant",
            AuditReadCapability::ACTION,
            &["document-retention-reviewer"][..],
        ),
    ] {
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    WorkflowAuthoringGrant::reference(),
                    key(grant),
                )
                .field(WorkflowGrantIdentityField::reference(), grant.to_owned())
                .field(WorkflowGrantActionField::reference(), action.to_owned())
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
            .expect("independent read grant seeds");
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                WorkflowGrantResource::reference(),
                &format!("{grant}-resource"),
                key::<WorkflowAuthoringGrant>(grant),
                key::<Document>("document-row-1"),
            ))
            .unwrap();
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                WorkflowGrantGrantor::reference(),
                &format!("{grant}-issuer"),
                key::<Principal>("document-retention-operator"),
                key::<WorkflowAuthoringGrant>(grant),
            ))
            .unwrap();
        for grantee in grantees {
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    WorkflowGrantGrantee::reference(),
                    &format!("{grant}-{grantee}"),
                    key::<Principal>(grantee),
                    key::<WorkflowAuthoringGrant>(grant),
                ))
                .unwrap();
        }
    }
}
fn key<E>(text: &str) -> WorthQueryApplicationEntityKey<DocumentRetentionSchema, E> {
    WorthQueryApplicationEntityKey::new(text).unwrap()
}
