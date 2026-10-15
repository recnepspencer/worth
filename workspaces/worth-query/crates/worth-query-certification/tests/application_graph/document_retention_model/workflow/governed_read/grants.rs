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
            "workflow-metadata-read-grant",
            MetadataReadCapability::ACTION,
            &["document-retention-operator"][..],
        ),
        (
            "workflow-audit-read-grant",
            AuditReadCapability::ACTION,
            &["document-retention-reviewer"][..],
        ),
    ] {
        seed_grant(
            graph,
            grant,
            action,
            "document-row-1",
            DOCUMENT_IDENTITY,
            grantees,
        );
    }
    // Genuine second-subject start authority; it does not grant second-subject Subject read.
    seed_grant(
        graph,
        "workflow-second-subject-start",
        "start-workflow-instance",
        "document-row-2",
        super::super::super::retention_entry::RELATED_DOCUMENT_IDENTITY,
        &["document-retention-operator"],
    );
    // A second independently authorized definition-read scope makes continuation
    // scope substitution observable even though both fresh capabilities admit.
    seed_grant(
        graph,
        "workflow-second-definition-read",
        DefinitionReadCapability::ACTION,
        "document-row-2",
        super::super::super::retention_entry::RELATED_DOCUMENT_IDENTITY,
        &["document-retention-operator"],
    );
}
fn seed_grant(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>,
    grant: &str,
    action: &str,
    document: &str,
    identity: &str,
    grantees: &[&str],
) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(WorkflowAuthoringGrant::reference(), key(grant))
                .field(WorkflowGrantIdentityField::reference(), grant.to_owned())
                .field(WorkflowGrantActionField::reference(), action.to_owned())
                .field(
                    WorkflowGrantPurposeField::reference(),
                    "reviewed-document".to_owned(),
                )
                .field(WorkflowGrantStatusField::reference(), "active".to_owned())
                .field(WorkflowGrantWorkflowField::reference(), identity.to_owned())
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
            key::<Document>(document),
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
fn key<E>(text: &str) -> WorthQueryApplicationEntityKey<DocumentRetentionSchema, E> {
    WorthQueryApplicationEntityKey::new(text).unwrap()
}
