#[path = "declaration/records.rs"]
mod storage_model;
pub use storage_model::*;
pub use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
    worth_query_operation_reads,
};
pub use worth_query_host::facade::declaration::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRelatedEntitySelector,
    ApplicationCapabilityRequest, ApplicationCapabilityRequestContext,
    ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
};
pub use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowSpec, ApplicationWorkflowSpecIdentity,
};
pub use worth_query_host::facade::declaration::application_schema::{
    ApplicationEncodedScalarValue, ApplicationSchemaDeclarationBuilder,
    StringApplicationValueBinding,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
pub use worth_query_host::facade::{
    worth_query_operation, worth_query_portable_type, worth_query_structured_value_binding,
};

pub use super::super::{
    retention_entry::{DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY},
    schema::{
        Document, DocumentIdentityField, DocumentRetentionSchema, Principal, SetRetentionInput,
        SetRetentionInputBinding,
    },
};

pub struct ReviewedDocumentWorkflow;

impl ApplicationWorkflowSpec for ReviewedDocumentWorkflow {
    fn observation_policy(
    ) -> worth_query_decl::facade::application_program::ApplicationWorkflowObservationPolicy {
        worth_query_decl::facade::application_program::ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy(
    ) -> worth_query_decl::facade::application_program::ApplicationWorkflowActorPolicy {
        worth_query_decl::facade::application_program::ApplicationWorkflowActorPolicy::for_approval::<
            Self, super::WorkflowApprovalBinding>(
                worth_query_decl::facade::application_capability::ApplicationWorkflowRecordedActorSource::BoundProposal)
    }

    type Schema = DocumentRetentionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity = ApplicationWorkflowSpecIdentity::new(
        "worth.query.certification.reviewed-document-workflow.v1",
    );
}

worth_query_capability_context!(pub WorkflowDefinitionAuthoringContext in DocumentRetentionSchema);
worth_query_capability_provenance!(pub WorkflowDefinitionAuthoringProvenance in DocumentRetentionSchema);
worth_query_capability!(pub WorkflowDefinitionAuthoringCapability in DocumentRetentionSchema);
worth_query_capability_context!(pub WorkflowInstanceStartContext in DocumentRetentionSchema);
worth_query_capability_provenance!(pub WorkflowInstanceStartProvenance in DocumentRetentionSchema);
worth_query_capability!(pub WorkflowInstanceStartCapability in DocumentRetentionSchema);

pub type WorkflowDefinitionAuthoringInput = SetRetentionInput;
pub type WorkflowDefinitionAuthoringInputBinding = SetRetentionInputBinding;
worth_query_operation!(pub WorkflowDefinitionAuthoringOperation for DocumentRetentionSchema, input WorkflowDefinitionAuthoringInputBinding);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct WorkflowInstanceStartInput {
    pub document_identity: String,
}

worth_query_portable_type!(WorkflowInstanceStartInput => "worth.query.certification.workflow-instance-start-input.v1");
worth_query_structured_value_binding!(pub WorkflowInstanceStartInputBinding for WorkflowInstanceStartInput {
    identity: "worth.query.certification.workflow-instance-start-input.v1"
});
worth_query_operation!(pub WorkflowInstanceStartOperation for DocumentRetentionSchema, input WorkflowInstanceStartInputBinding);

impl ApplicationCapabilityRequest<DocumentRetentionSchema, WorkflowDefinitionAuthoringCapability>
    for WorkflowDefinitionAuthoringInput
{
    type Scope = Document;
    type Context = WorkflowDefinitionAuthoringContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            DocumentRetentionSchema,
            Document,
            WorkflowDefinitionAuthoringContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::at("DocumentIdentityField"),
                encoded(self.identity.clone()),
            )
            .unwrap(),
            encoded("author-workflow-definition".to_owned()),
            encoded("reviewed-document".to_owned()),
            ApplicationCapabilityRequestContext::new(
                WorkflowDefinitionAuthoringContext::reference(),
            ),
        )
        .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            WorkflowGrantRelated::reference(),
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::at("DocumentIdentityField"),
                encoded(RELATED_DOCUMENT_IDENTITY.to_owned()),
            )
            .unwrap(),
        )))
    }
}

impl ApplicationCapabilityRequest<DocumentRetentionSchema, WorkflowInstanceStartCapability>
    for WorkflowInstanceStartInput
{
    type Scope = Document;
    type Context = WorkflowInstanceStartContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            DocumentRetentionSchema,
            Document,
            WorkflowInstanceStartContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::at("DocumentIdentityField"),
                encoded(self.document_identity.clone()),
            )
            .unwrap(),
            encoded("start-workflow-instance".to_owned()),
            encoded("reviewed-document".to_owned()),
            ApplicationCapabilityRequestContext::new(WorkflowInstanceStartContext::reference()),
        ))
    }
}

pub fn install_members(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .capability_context(WorkflowDefinitionAuthoringContext::reference())
        .capability_provenance(WorkflowDefinitionAuthoringProvenance::reference())
        .capability_context(WorkflowInstanceStartContext::reference())
        .capability_provenance(WorkflowInstanceStartProvenance::reference())
        .operation(
            WorkflowDefinitionAuthoringOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(WorkflowDefinitionAuthoringOperation::reference(), 512)
        .operation_read_field(
            WorkflowDefinitionAuthoringOperation::reference(),
            DocumentIdentityField::at("DocumentIdentityField"),
        )
        .operation(
            WorkflowInstanceStartOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(WorkflowInstanceStartOperation::reference(), 512)
        .operation_read_field(
            WorkflowInstanceStartOperation::reference(),
            DocumentIdentityField::at("DocumentIdentityField"),
        )
}

pub fn seed_authoring(graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                WorkflowAuthoringGrant::reference(),
                entity_key("workflow-authoring-grant"),
            )
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
                DOCUMENT_IDENTITY.to_owned(),
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
            WorkflowDocumentOwner::reference(),
            "workflow-document-owner",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<Document>("document-row-1"),
        ))
        .expect("workflow document ownership must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantResource::reference(),
            "workflow-grant-resource",
            entity_key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
            entity_key::<Document>("document-row-1"),
        ))
        .expect("workflow grant resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantRelated::reference(),
            "workflow-grant-related",
            entity_key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
            entity_key::<Document>("document-row-2"),
        ))
        .expect("workflow grant related subject must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "workflow-grant-grantor",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
        ))
        .expect("workflow grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "workflow-grant-grantee",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
        ))
        .expect("workflow grantee must seed");

    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                WorkflowAuthoringGrant::reference(),
                entity_key("workflow-instance-start-grant"),
            )
            .field(
                WorkflowGrantIdentityField::at("WorkflowGrantIdentityField"),
                "workflow-instance-start-grant".to_owned().into(),
            )
            .field(
                WorkflowGrantActionField::at("WorkflowGrantActionField"),
                "start-workflow-instance".to_owned(),
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
                DOCUMENT_IDENTITY.to_owned(),
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
        .expect("workflow instance-start grant must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantResource::reference(),
            "workflow-start-grant-resource",
            entity_key::<WorkflowAuthoringGrant>("workflow-instance-start-grant"),
            entity_key::<Document>("document-row-1"),
        ))
        .expect("workflow instance-start resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "workflow-start-grant-grantor",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-instance-start-grant"),
        ))
        .expect("workflow instance-start grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "workflow-start-grant-grantee",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-instance-start-grant"),
        ))
        .expect("workflow instance-start grantee must seed");
    super::advance::seed(graph);
}

pub fn encoded(value: String) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value)
        .expect("workflow authoring fixture text must encode")
}

pub fn entity_key<Entity>(
    value: &str,
) -> WorthQueryApplicationEntityKey<DocumentRetentionSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value).expect("workflow fixture key must be valid")
}
