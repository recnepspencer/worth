use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
    worth_query_operation_reads,
};
use worth_query_host::facade::declaration::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRelatedEntitySelector,
    ApplicationCapabilityRequest, ApplicationCapabilityRequestContext,
    ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowSpec, ApplicationWorkflowSpecIdentity,
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
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_operation,
    worth_query_portable_type, worth_query_relation, worth_query_structured_value_binding,
};

use super::super::{
    dimension_entry::{PART_IDENTITY, RELATED_PART_IDENTITY},
    schema::{
        BoundedDimensionSchema, Part, PartIdentityField, Principal, SetPartDimensionInput,
        SetPartDimensionInputBinding,
    },
};

pub struct ReviewedGeometryWorkflow;

impl ApplicationWorkflowSpec for ReviewedGeometryWorkflow {
    type Schema = BoundedDimensionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity = ApplicationWorkflowSpecIdentity::new(
        "worth.query.certification.reviewed-geometry-workflow.v1",
    );
}

worth_query_entity!(pub WorkflowAuthoringGrant for BoundedDimensionSchema);
worth_query_aspect!(pub WorkflowAuthoringGrantFacts for BoundedDimensionSchema, WorkflowAuthoringGrant; identity = AspectIdentity(0x91750601), revision = AspectContractRevision(1),);
worth_query_field!(pub WorkflowGrantIdentityField for BoundedDimensionSchema, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub WorkflowGrantActionField for BoundedDimensionSchema, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts: String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub WorkflowGrantPurposeField for BoundedDimensionSchema, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts: String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub WorkflowGrantStatusField for BoundedDimensionSchema, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts: String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub WorkflowGrantWorkflowField for BoundedDimensionSchema, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts: String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub WorkflowGrantNotBeforeField for BoundedDimensionSchema, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts: u64 => worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub WorkflowGrantNotAfterField for BoundedDimensionSchema, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts: u64 => worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub WorkflowGrantDelegationLimitField for BoundedDimensionSchema, WorkflowAuthoringGrant, WorkflowAuthoringGrantFacts: u64 => worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding, read_write, no_equality);

worth_query_relation!(pub WorkflowPartOwner in BoundedDimensionSchema, Principal => Part; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub WorkflowGrantResource in BoundedDimensionSchema, WorkflowAuthoringGrant => Part; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub WorkflowGrantRelated in BoundedDimensionSchema, WorkflowAuthoringGrant => Part; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub WorkflowGrantParent in BoundedDimensionSchema, WorkflowAuthoringGrant => WorkflowAuthoringGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub WorkflowGrantGrantor in BoundedDimensionSchema, Principal => WorkflowAuthoringGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub WorkflowGrantGrantee in BoundedDimensionSchema, Principal => WorkflowAuthoringGrant; integrity = same_context_unbounded_retain_dangling);

worth_query_capability_context!(pub WorkflowDefinitionAuthoringContext in BoundedDimensionSchema);
worth_query_capability_provenance!(pub WorkflowDefinitionAuthoringProvenance in BoundedDimensionSchema);
worth_query_capability!(pub WorkflowDefinitionAuthoringCapability in BoundedDimensionSchema);
worth_query_capability_context!(pub WorkflowInstanceStartContext in BoundedDimensionSchema);
worth_query_capability_provenance!(pub WorkflowInstanceStartProvenance in BoundedDimensionSchema);
worth_query_capability!(pub WorkflowInstanceStartCapability in BoundedDimensionSchema);

pub type WorkflowDefinitionAuthoringInput = SetPartDimensionInput;
pub type WorkflowDefinitionAuthoringInputBinding = SetPartDimensionInputBinding;
worth_query_operation!(pub WorkflowDefinitionAuthoringOperation for BoundedDimensionSchema, input WorkflowDefinitionAuthoringInputBinding);
worth_query_operation_reads!(WorkflowDefinitionAuthoringOperation => [PartIdentityField]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowInstanceStartInput {
    pub part_identity: String,
}

worth_query_portable_type!(WorkflowInstanceStartInput => "worth.query.certification.workflow-instance-start-input.v1");
worth_query_structured_value_binding!(pub WorkflowInstanceStartInputBinding for WorkflowInstanceStartInput {
    identity: "worth.query.certification.workflow-instance-start-input.v1"
});
worth_query_operation!(pub WorkflowInstanceStartOperation for BoundedDimensionSchema, input WorkflowInstanceStartInputBinding);
worth_query_operation_reads!(WorkflowInstanceStartOperation => [PartIdentityField]);

impl ApplicationCapabilityRequest<BoundedDimensionSchema, WorkflowDefinitionAuthoringCapability>
    for WorkflowDefinitionAuthoringInput
{
    type Scope = Part;
    type Context = WorkflowDefinitionAuthoringContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            BoundedDimensionSchema,
            Part,
            WorkflowDefinitionAuthoringContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                PartIdentityField::reference(),
                encoded(self.identity.clone()),
            ),
            encoded("author-workflow-definition".to_owned()),
            encoded("reviewed-geometry".to_owned()),
            ApplicationCapabilityRequestContext::new(
                WorkflowDefinitionAuthoringContext::reference(),
            ),
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

impl ApplicationCapabilityRequest<BoundedDimensionSchema, WorkflowInstanceStartCapability>
    for WorkflowInstanceStartInput
{
    type Scope = Part;
    type Context = WorkflowInstanceStartContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            BoundedDimensionSchema,
            Part,
            WorkflowInstanceStartContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                PartIdentityField::reference(),
                encoded(self.part_identity.clone()),
            ),
            encoded("start-workflow-instance".to_owned()),
            encoded("reviewed-geometry".to_owned()),
            ApplicationCapabilityRequestContext::new(WorkflowInstanceStartContext::reference()),
        ))
    }
}

pub(super) fn install_members(
    schema: ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema>,
) -> ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema> {
    schema
        .entity(WorkflowAuthoringGrant::reference())
        .aspect(
            WorkflowAuthoringGrant::reference(),
            WorkflowAuthoringGrantFacts::reference(),
        )
        .field(
            WorkflowAuthoringGrant::reference(),
            WorkflowGrantIdentityField::reference(),
        )
        .field(
            WorkflowAuthoringGrant::reference(),
            WorkflowGrantActionField::reference(),
        )
        .field(
            WorkflowAuthoringGrant::reference(),
            WorkflowGrantPurposeField::reference(),
        )
        .field(
            WorkflowAuthoringGrant::reference(),
            WorkflowGrantStatusField::reference(),
        )
        .field(
            WorkflowAuthoringGrant::reference(),
            WorkflowGrantWorkflowField::reference(),
        )
        .field(
            WorkflowAuthoringGrant::reference(),
            WorkflowGrantNotBeforeField::reference(),
        )
        .field(
            WorkflowAuthoringGrant::reference(),
            WorkflowGrantNotAfterField::reference(),
        )
        .field(
            WorkflowAuthoringGrant::reference(),
            WorkflowGrantDelegationLimitField::reference(),
        )
        .relation(
            WorkflowPartOwner::reference(),
            Principal::reference(),
            Part::reference(),
        )
        .relation(
            WorkflowGrantResource::reference(),
            WorkflowAuthoringGrant::reference(),
            Part::reference(),
        )
        .relation(
            WorkflowGrantRelated::reference(),
            WorkflowAuthoringGrant::reference(),
            Part::reference(),
        )
        .relation(
            WorkflowGrantParent::reference(),
            WorkflowAuthoringGrant::reference(),
            WorkflowAuthoringGrant::reference(),
        )
        .relation(
            WorkflowGrantGrantor::reference(),
            Principal::reference(),
            WorkflowAuthoringGrant::reference(),
        )
        .relation(
            WorkflowGrantGrantee::reference(),
            Principal::reference(),
            WorkflowAuthoringGrant::reference(),
        )
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
        .operation_decision_fact_budget(WorkflowDefinitionAuthoringOperation::reference(), 512)
        .operation_projection_work_budget(WorkflowDefinitionAuthoringOperation::reference(), 512)
        .operation_read_field(
            WorkflowDefinitionAuthoringOperation::reference(),
            PartIdentityField::reference(),
        )
        .operation(
            WorkflowInstanceStartOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(WorkflowInstanceStartOperation::reference(), 256)
        .operation_projection_work_budget(WorkflowInstanceStartOperation::reference(), 512)
        .operation_read_field(
            WorkflowInstanceStartOperation::reference(),
            PartIdentityField::reference(),
        )
}

pub fn seed_authoring(graph: &mut WorthQueryPrimaryGraphBootstrap<BoundedDimensionSchema>) {
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                WorkflowAuthoringGrant::reference(),
                entity_key("workflow-authoring-grant"),
            )
            .field(
                WorkflowGrantIdentityField::reference(),
                "workflow-authoring-grant".to_owned(),
            )
            .field(
                WorkflowGrantActionField::reference(),
                "author-workflow-definition".to_owned(),
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
        .expect("workflow authoring grant must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowPartOwner::reference(),
            "workflow-part-owner",
            entity_key::<Principal>("bounded-dimension-operator"),
            entity_key::<Part>("part-row-1"),
        ))
        .expect("workflow part ownership must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantResource::reference(),
            "workflow-grant-resource",
            entity_key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
            entity_key::<Part>("part-row-1"),
        ))
        .expect("workflow grant resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantRelated::reference(),
            "workflow-grant-related",
            entity_key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
            entity_key::<Part>("part-row-2"),
        ))
        .expect("workflow grant related subject must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "workflow-grant-grantor",
            entity_key::<Principal>("bounded-dimension-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-authoring-grant"),
        ))
        .expect("workflow grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "workflow-grant-grantee",
            entity_key::<Principal>("bounded-dimension-operator"),
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
                WorkflowGrantIdentityField::reference(),
                "workflow-instance-start-grant".to_owned(),
            )
            .field(
                WorkflowGrantActionField::reference(),
                "start-workflow-instance".to_owned(),
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
        .expect("workflow instance-start grant must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantResource::reference(),
            "workflow-start-grant-resource",
            entity_key::<WorkflowAuthoringGrant>("workflow-instance-start-grant"),
            entity_key::<Part>("part-row-1"),
        ))
        .expect("workflow instance-start resource must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantor::reference(),
            "workflow-start-grant-grantor",
            entity_key::<Principal>("bounded-dimension-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-instance-start-grant"),
        ))
        .expect("workflow instance-start grantor must seed");
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            WorkflowGrantGrantee::reference(),
            "workflow-start-grant-grantee",
            entity_key::<Principal>("bounded-dimension-operator"),
            entity_key::<WorkflowAuthoringGrant>("workflow-instance-start-grant"),
        ))
        .expect("workflow instance-start grantee must seed");
    super::advance::seed(graph);
}

fn encoded(value: String) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value)
        .expect("workflow authoring fixture text must encode")
}

fn entity_key<Entity>(
    value: &str,
) -> WorthQueryApplicationEntityKey<BoundedDimensionSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value).expect("workflow fixture key must be valid")
}
