use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
    worth_query_operation_reads,
};
use worth_query_host::facade::declaration::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRelatedEntitySelector,
    ApplicationCapabilityRequest, ApplicationCapabilityRequestContext,
    ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
};
use worth_query_host::facade::declaration::application_schema::{
    ApplicationEncodedScalarValue, ApplicationSchemaDeclarationBuilder,
    StringApplicationValueBinding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
use worth_query_host::facade::{
    worth_query_operation, worth_query_portable_type, worth_query_structured_value_binding,
};

use super::super::super::{
    retention_entry::{DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY},
    schema::{Document, DocumentIdentityField, DocumentRetentionSchema, Principal},
};
use super::super::declaration::{
    WorkflowAuthoringGrant, WorkflowGrantActionField, WorkflowGrantDelegationLimitField,
    WorkflowGrantGrantee, WorkflowGrantGrantor, WorkflowGrantIdentityField,
    WorkflowGrantNotAfterField, WorkflowGrantNotBeforeField, WorkflowGrantPurposeField,
    WorkflowGrantRelated, WorkflowGrantResource, WorkflowGrantStatusField,
    WorkflowGrantWorkflowField,
};

worth_query_capability_context!(pub WorkflowAdvanceContext in DocumentRetentionSchema);
worth_query_capability_provenance!(pub WorkflowAdvanceProvenance in DocumentRetentionSchema);
worth_query_capability!(pub WorkflowAdvanceCapability in DocumentRetentionSchema);
worth_query_capability!(pub WorkflowApprovalCapability in DocumentRetentionSchema);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct WorkflowAdvanceInput {
    pub document_identity: String,
}

worth_query_portable_type!(WorkflowAdvanceInput => "worth.query.certification.workflow-advance-input.v1");
worth_query_structured_value_binding!(pub WorkflowAdvanceInputBinding for WorkflowAdvanceInput {
    identity: "worth.query.certification.workflow-advance-input.v1"
});
worth_query_operation!(pub WorkflowAdvanceOperation for DocumentRetentionSchema, input WorkflowAdvanceInputBinding);
worth_query_operation_reads!(WorkflowAdvanceOperation => [DocumentIdentityField]);

impl ApplicationCapabilityRequest<DocumentRetentionSchema, WorkflowAdvanceCapability>
    for WorkflowAdvanceInput
{
    type Scope = Document;
    type Context = WorkflowAdvanceContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            DocumentRetentionSchema,
            Document,
            WorkflowAdvanceContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::reference(),
                encoded(self.document_identity.clone()),
            ),
            encoded("advance-workflow-instance".to_owned()),
            encoded("reviewed-document".to_owned()),
            ApplicationCapabilityRequestContext::new(WorkflowAdvanceContext::reference()),
        )
        .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            WorkflowGrantRelated::reference(),
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::reference(),
                encoded(RELATED_DOCUMENT_IDENTITY.to_owned()),
            ),
        )))
    }
}

impl ApplicationCapabilityRequest<DocumentRetentionSchema, WorkflowApprovalCapability>
    for WorkflowAdvanceInput
{
    type Scope = Document;
    type Context = WorkflowAdvanceContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            DocumentRetentionSchema,
            Document,
            WorkflowAdvanceContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::reference(),
                encoded(self.document_identity.clone()),
            ),
            encoded("approve-workflow-transition".to_owned()),
            encoded("reviewed-document".to_owned()),
            ApplicationCapabilityRequestContext::new(WorkflowAdvanceContext::reference()),
        ))
    }
}

pub(super) fn install_members(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .capability_context(WorkflowAdvanceContext::reference())
        .capability_provenance(WorkflowAdvanceProvenance::reference())
        .operation(
            WorkflowAdvanceOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(WorkflowAdvanceOperation::reference(), 512)
        .operation_read_field(
            WorkflowAdvanceOperation::reference(),
            DocumentIdentityField::reference(),
        )
}

pub(super) fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                WorkflowAuthoringGrant::reference(),
                entity_key("workflow-advance-grant"),
            )
            .field(
                WorkflowGrantIdentityField::reference(),
                "workflow-advance-grant".to_owned(),
            )
            .field(
                WorkflowGrantActionField::reference(),
                "advance-workflow-instance".to_owned(),
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
        .expect("workflow advance grant must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantResource::reference(),
            "workflow-advance-grant-resource",
            entity_key::<WorkflowAuthoringGrant>("workflow-advance-grant"),
            entity_key::<Document>("document-row-1"),
        ))
        .expect("workflow advance resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantRelated::reference(),
            "workflow-advance-grant-related",
            entity_key::<WorkflowAuthoringGrant>("workflow-advance-grant"),
            entity_key::<Document>("document-row-2"),
        ))
        .expect("workflow advance related subject must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "workflow-advance-grant-grantor",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-advance-grant"),
        ))
        .expect("workflow advance grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "workflow-advance-grant-grantee",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-advance-grant"),
        ))
        .expect("workflow advance grantee must seed");
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                WorkflowAuthoringGrant::reference(),
                entity_key("workflow-approval-grant"),
            )
            .field(
                WorkflowGrantIdentityField::reference(),
                "workflow-approval-grant".to_owned(),
            )
            .field(
                WorkflowGrantActionField::reference(),
                "approve-workflow-transition".to_owned(),
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
        .expect("workflow approval grant must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantResource::reference(),
            "workflow-approval-grant-resource",
            entity_key::<WorkflowAuthoringGrant>("workflow-approval-grant"),
            entity_key::<Document>("document-row-1"),
        ))
        .expect("workflow approval resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "workflow-approval-grant-grantor",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-approval-grant"),
        ))
        .expect("workflow approval grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "workflow-approval-grant-grantee",
            entity_key::<Principal>("document-retention-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-approval-grant"),
        ))
        .expect("workflow approval grantee must seed");
}

fn encoded(value: String) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value)
        .expect("workflow advance fixture text must encode")
}

fn entity_key<Entity>(
    value: &str,
) -> WorthQueryApplicationEntityKey<DocumentRetentionSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value).expect("workflow advance fixture key must be valid")
}
