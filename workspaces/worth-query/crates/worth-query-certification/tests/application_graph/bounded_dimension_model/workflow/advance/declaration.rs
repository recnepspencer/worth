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
    dimension_entry::{PART_IDENTITY, RELATED_PART_IDENTITY},
    schema::{BoundedDimensionSchema, Part, PartIdentityField, Principal},
};
use super::super::declaration::{
    WorkflowAuthoringGrant, WorkflowGrantActionField, WorkflowGrantDelegationLimitField,
    WorkflowGrantGrantee, WorkflowGrantGrantor, WorkflowGrantNotAfterField,
    WorkflowGrantNotBeforeField, WorkflowGrantPurposeField, WorkflowGrantRelated,
    WorkflowGrantResource, WorkflowGrantStatusField, WorkflowGrantWorkflowField,
};

worth_query_capability_context!(pub WorkflowAdvanceContext in BoundedDimensionSchema);
worth_query_capability_provenance!(pub WorkflowAdvanceProvenance in BoundedDimensionSchema);
worth_query_capability!(pub WorkflowAdvanceCapability in BoundedDimensionSchema);
worth_query_capability!(pub WorkflowApprovalCapability in BoundedDimensionSchema);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowAdvanceInput {
    pub part_identity: String,
}

worth_query_portable_type!(WorkflowAdvanceInput => "worth.query.certification.workflow-advance-input.v1");
worth_query_structured_value_binding!(pub WorkflowAdvanceInputBinding for WorkflowAdvanceInput {
    identity: "worth.query.certification.workflow-advance-input.v1"
});
worth_query_operation!(pub WorkflowAdvanceOperation for BoundedDimensionSchema, input WorkflowAdvanceInputBinding);
worth_query_operation_reads!(WorkflowAdvanceOperation => [PartIdentityField]);

impl ApplicationCapabilityRequest<BoundedDimensionSchema, WorkflowAdvanceCapability>
    for WorkflowAdvanceInput
{
    type Scope = Part;
    type Context = WorkflowAdvanceContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            BoundedDimensionSchema,
            Part,
            WorkflowAdvanceContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                PartIdentityField::reference(),
                encoded(self.part_identity.clone()),
            ),
            encoded("advance-workflow-instance".to_owned()),
            encoded("reviewed-geometry".to_owned()),
            ApplicationCapabilityRequestContext::new(WorkflowAdvanceContext::reference()),
        )
        .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            WorkflowGrantRelated::reference(),
            ApplicationCapabilityEntitySelector::new(
                PartIdentityField::reference(),
                encoded(RELATED_PART_IDENTITY.to_owned()),
            ),
        )))
    }
}

impl ApplicationCapabilityRequest<BoundedDimensionSchema, WorkflowApprovalCapability>
    for WorkflowAdvanceInput
{
    type Scope = Part;
    type Context = WorkflowAdvanceContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            BoundedDimensionSchema,
            Part,
            WorkflowAdvanceContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                PartIdentityField::reference(),
                encoded(self.part_identity.clone()),
            ),
            encoded("approve-workflow-transition".to_owned()),
            encoded("reviewed-geometry".to_owned()),
            ApplicationCapabilityRequestContext::new(WorkflowAdvanceContext::reference()),
        ))
    }
}

pub(super) fn install_members(
    schema: ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema>,
) -> ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema> {
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
        .operation_decision_fact_budget(WorkflowAdvanceOperation::reference(), 2048)
        .operation_projection_work_budget(WorkflowAdvanceOperation::reference(), 512)
        .operation_read_field(
            WorkflowAdvanceOperation::reference(),
            PartIdentityField::reference(),
        )
}

pub(super) fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<BoundedDimensionSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                WorkflowAuthoringGrant::reference(),
                entity_key("workflow-advance-grant"),
            )
            .field(
                WorkflowGrantActionField::reference(),
                "advance-workflow-instance".to_owned(),
            )
            .field(
                WorkflowGrantPurposeField::reference(),
                "reviewed-geometry".to_owned(),
            )
            .field(WorkflowGrantStatusField::reference(), "active".to_owned())
            .field(
                WorkflowGrantWorkflowField::reference(),
                PART_IDENTITY.to_owned(),
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
            entity_key::<Part>("part-row-1"),
        ))
        .expect("workflow advance resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantRelated::reference(),
            "workflow-advance-grant-related",
            entity_key::<WorkflowAuthoringGrant>("workflow-advance-grant"),
            entity_key::<Part>("part-row-2"),
        ))
        .expect("workflow advance related subject must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "workflow-advance-grant-grantor",
            entity_key::<Principal>("bounded-dimension-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-advance-grant"),
        ))
        .expect("workflow advance grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "workflow-advance-grant-grantee",
            entity_key::<Principal>("bounded-dimension-operator"),
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
                WorkflowGrantActionField::reference(),
                "approve-workflow-transition".to_owned(),
            )
            .field(
                WorkflowGrantPurposeField::reference(),
                "reviewed-geometry".to_owned(),
            )
            .field(WorkflowGrantStatusField::reference(), "active".to_owned())
            .field(
                WorkflowGrantWorkflowField::reference(),
                PART_IDENTITY.to_owned(),
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
            entity_key::<Part>("part-row-1"),
        ))
        .expect("workflow approval resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "workflow-approval-grant-grantor",
            entity_key::<Principal>("bounded-dimension-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-approval-grant"),
        ))
        .expect("workflow approval grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "workflow-approval-grant-grantee",
            entity_key::<Principal>("bounded-dimension-operator"),
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
) -> WorthQueryApplicationEntityKey<BoundedDimensionSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value).expect("workflow advance fixture key must be valid")
}
