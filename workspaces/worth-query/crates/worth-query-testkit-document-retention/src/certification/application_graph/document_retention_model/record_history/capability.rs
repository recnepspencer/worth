//! Two authored purposes share the existing real grant entity and native capability engine.
pub use super::super::workflow::{
    declaration::*,
    governed_read::{WorkflowReadContext, WorkflowReadProvenance},
};
pub use super::*;
pub use worth_query_decl::facade::{
    application_capability::*, worth_query_capability, worth_query_operation,
    worth_query_operation_reads, worth_query_structured_value_binding,
};

worth_query_capability!(pub PayloadCapability in DocumentRetentionSchema);
worth_query_capability!(pub MetadataCapability in DocumentRetentionSchema);
#[derive(Clone, Debug, serde::Serialize)]
pub struct Input {
    pub identity: String,
}
worth_query_portable_type!(Input => "worth.query.certification.record-history.capability-input.v1");
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "worth.query.certification.record-history.capability-input.v1" });
worth_query_operation!(pub PayloadRead for DocumentRetentionSchema, input InputBinding);
worth_query_operation!(pub MetadataRead for DocumentRetentionSchema, input InputBinding);
worth_query_operation_reads!(PayloadRead => [DocumentIdentityField]);
worth_query_operation_reads!(MetadataRead => [DocumentIdentityField]);

pub trait Role: ApplicationCapabilityMarkerIdentity<Schema = DocumentRetentionSchema> {
    const ACTION: &'static str;
}
impl Role for PayloadCapability {
    const ACTION: &'static str = "read-record-history-subject";
}
impl Role for MetadataCapability {
    const ACTION: &'static str = "read-record-history-metadata";
}

impl<Capability: Role> ApplicationCapabilityRequest<DocumentRetentionSchema, Capability> for Input {
    type Scope = Document;
    type Context = WorkflowReadContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            DocumentRetentionSchema,
            Document,
            WorkflowReadContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::reference(),
                text(&self.identity),
            ),
            text(Capability::ACTION),
            encoded(),
            ApplicationCapabilityRequestContext::new(WorkflowReadContext::reference()),
        )
        .field(encoded()))
    }
}

pub fn encoded() -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    text("reviewed-document")
}
pub fn text(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
}

pub fn contract<C: Role, O>(
    capability: ApplicationCapabilityRef<DocumentRetentionSchema, C>,
    operation: ApplicationOperationRef<DocumentRetentionSchema, O, Input>,
) -> ApplicationCapabilityContract<DocumentRetentionSchema, C, O, Input>
where
    O: ApplicationOperationMarkerIdentity<DocumentRetentionSchema, InputBinding = InputBinding>,
{
    let ordinary = super::super::workflow::contract::composition();
    let composition = ApplicationCapabilityComposition::new(
        ordinary.decision().clone(),
        ordinary.actors().clone(),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(
                        WorkflowGrantPurposeField::reference(),
                        [encoded()],
                    ),
                ]),
            ]),
        ),
    );
    ApplicationCapabilityContractBuilder::new(
        capability,
        operation,
        WorkflowAuthoringGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantActionField::reference(),
            text(C::ACTION),
        ),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(WorkflowGrantPurposeField::reference()),
        ApplicationCapabilityValueBinding::new(WorkflowGrantPurposeField::reference(), encoded()),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::reference(),
                text("active"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantWorkflowField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    DocumentIdentityField::reference(),
                ),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotBeforeField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    WorkflowGrantNotAfterField::reference(),
                ),
            ),
        ),
        WorkflowReadContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(
            WorkflowGrantDelegationLimitField::reference(),
        ),
        WorkflowReadProvenance::reference(),
    ))
    .composition(composition)
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let payload = PayloadRead::reference();
    let metadata = MetadataRead::reference();
    schema
        .operation(
            payload
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(payload, DocumentIdentityField::reference())
        .capability(contract(PayloadCapability::reference(), payload))
        .operation(
            metadata
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(metadata, DocumentIdentityField::reference())
        .capability(contract(MetadataCapability::reference(), metadata))
}
