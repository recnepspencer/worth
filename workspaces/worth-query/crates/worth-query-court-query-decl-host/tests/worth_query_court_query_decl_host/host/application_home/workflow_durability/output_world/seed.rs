//! Bootstrap gives identity, grants and relationships their real Native provenance.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase;
pub fn seed(
    _: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(&installed.principal_binding(authentication()).unwrap(), WorthQueryApplicationPrincipalKey::new("document-retention-operator").unwrap(), 1u64, document::operator_identity::operator_identity(), worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
    for (row, identity) in [
        (
            "document-row-1",
            document::retention_entry::DOCUMENT_IDENTITY,
        ),
        (
            "document-row-2",
            document::retention_entry::RELATED_DOCUMENT_IDENTITY,
        ),
    ] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(record(), key(row))
                .field(
                    DocumentIdentityField::at("DocumentIdentityField"),
                    identity.to_owned(),
                )
                .field(DocumentRetentionField::at("DocumentRetentionField"), 7u64),
        )?;
    }
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(grant(), key("workflow-authoring-grant"))
                .field(
                    WorkflowGrantIdentityField::at("WorkflowGrantIdentityField"),
                    "workflow-authoring-grant".to_owned().into(),
                )
                .field(
                    WorkflowGrantActionField::at("WorkflowGrantActionField"),
                    "author-workflow-definition".to_owned(),
                )
                .field(
                    WorkflowGrantPurposeField::at("WorkflowGrantPurposeField"),
                    "reviewed-document".to_owned(),
                )
                .field(
                    WorkflowGrantStatusField::at("WorkflowGrantStatusField"),
                    "active".to_owned(),
                )
                .field(
                    WorkflowGrantWorkflowField::at("WorkflowGrantWorkflowField"),
                    document::retention_entry::DOCUMENT_IDENTITY.to_owned(),
                )
                .field(
                    WorkflowGrantNotBeforeField::at("WorkflowGrantNotBeforeField"),
                    0_u64,
                )
                .field(
                    WorkflowGrantNotAfterField::at("WorkflowGrantNotAfterField"),
                    u64::MAX,
                )
                .field(
                    WorkflowGrantDelegationLimitField::at("WorkflowGrantDelegationLimitField"),
                    0_u64,
                ),
        )
        .expect("workflow authoring grant must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            owner(),
            "workflow-document-owner",
            key::<Principal>("document-retention-operator"),
            key::<Document>("document-row-1"),
        ))
        .expect("workflow document ownership must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            resource(),
            "workflow-grant-resource",
            key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
            key::<Document>("document-row-1"),
        ))
        .expect("workflow grant resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            related(),
            "workflow-grant-related",
            key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
            key::<Document>("document-row-2"),
        ))
        .expect("workflow grant related subject must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            grantor(),
            "workflow-grant-grantor",
            key::<Principal>("document-retention-operator"),
            key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
        ))
        .expect("workflow grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            grantee(),
            "workflow-grant-grantee",
            key::<Principal>("document-retention-operator"),
            key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
        ))
        .expect("workflow grantee must seed");

    Ok(())
}
fn key<E>(name: &str) -> WorthQueryApplicationEntityKey<Schema, E> {
    WorthQueryApplicationEntityKey::new(name).unwrap()
}
