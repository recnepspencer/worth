use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRequest,
    ApplicationCapabilityRequestContext, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, StringApplicationValueBinding,
};
use worth_query_declaration::{
    worth_query_application_schema, worth_query_aspect, worth_query_capability,
    worth_query_capability_context, worth_query_capability_context_entity_slot,
    worth_query_capability_provenance, worth_query_entity, worth_query_field,
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_principal_binding, worth_query_relation,
};

worth_query_application_schema! {
    pub schema PublicationAuthorizationSchema {
        owner: publication_authorization_proof,
        version: (1, 0),
        members: |schema| {
            let schema = schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(Resource::reference())
                .entity(CapabilityGrant::reference())
                .entity(ActionRecord::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .aspect(Resource::reference(), ResourceFacts::reference())
                .aspect(CapabilityGrant::reference(), GrantFacts::reference())
                .aspect(ActionRecord::reference(), ActionRecordFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(Resource::reference(), ResourceIdentityField::reference())
                .field(Resource::reference(), ResourceWorkflowField::reference())
                .field(Resource::reference(), ResourceLabelField::reference())
                .field(CapabilityGrant::reference(), GrantIdentityField::reference())
                .field(CapabilityGrant::reference(), GrantActionField::reference())
                .field(CapabilityGrant::reference(), GrantPurposeField::reference())
                .field(CapabilityGrant::reference(), GrantStatusField::reference())
                .field(CapabilityGrant::reference(), GrantWorkflowField::reference())
                .field(CapabilityGrant::reference(), GrantNotBeforeField::reference())
                .field(CapabilityGrant::reference(), GrantNotAfterField::reference())
                .field(CapabilityGrant::reference(), GrantDelegationLimitField::reference())
                .field(ActionRecord::reference(), ActionRecordIdentityField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .relation(ResourceOwner::reference(), Principal::reference(), Resource::reference())
                .relation(GrantGrantee::reference(), Principal::reference(), CapabilityGrant::reference())
                .relation(GrantGrantor::reference(), Principal::reference(), CapabilityGrant::reference())
                .relation(GrantResource::reference(), CapabilityGrant::reference(), Resource::reference())
                .relation(GrantParent::reference(), CapabilityGrant::reference(), CapabilityGrant::reference())
                .relation(ExplicitDeny::reference(), Principal::reference(), Resource::reference())
                .relation(ConflictingActor::reference(), Principal::reference(), Resource::reference())
                .relation(RequestActor::reference(), Principal::reference(), ActionRecord::reference())
                .relation(PriorActor::reference(), Principal::reference(), ActionRecord::reference())
                .relation(ActionResource::reference(), ActionRecord::reference(), Resource::reference())
                .principal_binding(PublicationIdentityBinding::reference())
                .capability_context(PublicationRequestContext::reference())
                .capability_context_entity_slot(RequestActorSlot::reference())
                .capability_context_entity_slot(PriorActorSlot::reference())
                .capability_provenance(PublicationCapabilityProvenance::reference())
                .operation(
                    PublicationOperation::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_decision_fact_budget(PublicationOperation::reference(), 1)
                .operation_projection_work_budget(PublicationOperation::reference(), 16)
                .operation_read_field(PublicationOperation::reference(), ResourceLabelField::reference())
                .operation_write(PublicationOperation::reference(), ResourceLabelField::reference());
            super::contract::install(schema)
        }
    }
}

worth_query_entity!(pub ExternalMapping for PublicationAuthorizationSchema);
worth_query_entity!(pub Principal for PublicationAuthorizationSchema);
worth_query_entity!(pub Resource for PublicationAuthorizationSchema);
worth_query_entity!(pub CapabilityGrant for PublicationAuthorizationSchema);
worth_query_entity!(pub ActionRecord for PublicationAuthorizationSchema);

worth_query_aspect!(pub ExternalIdentity for PublicationAuthorizationSchema, ExternalMapping; identity = AspectIdentity(0x9161104c), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts for PublicationAuthorizationSchema, Principal; identity = AspectIdentity(0x9161104d), revision = AspectContractRevision(1),);
worth_query_aspect!(pub ResourceFacts for PublicationAuthorizationSchema, Resource; identity = AspectIdentity(0x9161104e), revision = AspectContractRevision(1),);
worth_query_aspect!(pub GrantFacts for PublicationAuthorizationSchema, CapabilityGrant; identity = AspectIdentity(0x9161104f), revision = AspectContractRevision(1),);
worth_query_aspect!(pub ActionRecordFacts for PublicationAuthorizationSchema, ActionRecord; identity = AspectIdentity(0x91611050), revision = AspectContractRevision(1),);

worth_query_field!(pub ExternalIdentityField for PublicationAuthorizationSchema, ExternalMapping, ExternalIdentity: worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity => worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for PublicationAuthorizationSchema, ExternalMapping, ExternalIdentity: worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus => worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for PublicationAuthorizationSchema, Principal, PrincipalFacts: u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub ResourceIdentityField for PublicationAuthorizationSchema, Resource, ResourceFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ResourceWorkflowField for PublicationAuthorizationSchema, Resource, ResourceFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub ResourceLabelField for PublicationAuthorizationSchema, Resource, ResourceFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub GrantIdentityField for PublicationAuthorizationSchema, CapabilityGrant, GrantFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantActionField for PublicationAuthorizationSchema, CapabilityGrant, GrantFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantPurposeField for PublicationAuthorizationSchema, CapabilityGrant, GrantFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantStatusField for PublicationAuthorizationSchema, CapabilityGrant, GrantFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub GrantWorkflowField for PublicationAuthorizationSchema, CapabilityGrant, GrantFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub GrantNotBeforeField for PublicationAuthorizationSchema, CapabilityGrant, GrantFacts: u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub GrantNotAfterField for PublicationAuthorizationSchema, CapabilityGrant, GrantFacts: u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub GrantDelegationLimitField for PublicationAuthorizationSchema, CapabilityGrant, GrantFacts: u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ActionRecordIdentityField for PublicationAuthorizationSchema, ActionRecord, ActionRecordFacts: String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_only, equality);

worth_query_relation!(pub MappingTarget in PublicationAuthorizationSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ResourceOwner in PublicationAuthorizationSchema, Principal => Resource; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantee in PublicationAuthorizationSchema, Principal => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantor in PublicationAuthorizationSchema, Principal => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantResource in PublicationAuthorizationSchema, CapabilityGrant => Resource; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantParent in PublicationAuthorizationSchema, CapabilityGrant => CapabilityGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ExplicitDeny in PublicationAuthorizationSchema, Principal => Resource; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ConflictingActor in PublicationAuthorizationSchema, Principal => Resource; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub RequestActor in PublicationAuthorizationSchema, Principal => ActionRecord; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub PriorActor in PublicationAuthorizationSchema, Principal => ActionRecord; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ActionResource in PublicationAuthorizationSchema, ActionRecord => Resource; integrity = same_context_unbounded_retain_dangling);

worth_query_principal_binding!(pub PublicationIdentityBinding in PublicationAuthorizationSchema, mapping ExternalMapping { identity: ExternalIdentityField, status: MappingStatusField, target: MappingTarget => Principal, principal_identity: PrincipalIdentityField });
worth_query_capability_context!(pub PublicationRequestContext in PublicationAuthorizationSchema);
worth_query_capability_context_entity_slot!(pub RequestActorSlot in PublicationAuthorizationSchema, PublicationRequestContext => ActionRecord);
worth_query_capability_context_entity_slot!(pub PriorActorSlot in PublicationAuthorizationSchema, PublicationRequestContext => ActionRecord);
worth_query_capability_provenance!(pub PublicationCapabilityProvenance in PublicationAuthorizationSchema);
worth_query_capability!(pub PublicationCapability in PublicationAuthorizationSchema);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationInput;
worth_query_declaration::worth_query_portable_type!(PublicationInput => "worth.query.test.publication.input.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub PublicationInputBinding for PublicationInput { identity: "worth.query.test.publication.input.v1" });

worth_query_operation!(pub PublicationOperation for PublicationAuthorizationSchema, input PublicationInputBinding);
worth_query_operation_reads!(PublicationOperation => [ResourceLabelField]);
worth_query_operation_writes!(PublicationOperation => [ResourceLabelField]);

impl ApplicationCapabilityRequest<PublicationAuthorizationSchema, PublicationCapability>
    for PublicationInput
{
    type Scope = Resource;
    type Context = PublicationRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            PublicationAuthorizationSchema,
            Resource,
            PublicationRequestContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                ResourceIdentityField::reference(),
                encoded_string("resource-1"),
            ),
            encoded_string("inspect"),
            encoded_string("publication-proof"),
            ApplicationCapabilityRequestContext::new(PublicationRequestContext::reference())
                .entity(
                    RequestActorSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        ActionRecordIdentityField::reference(),
                        encoded_string("selected-request"),
                    ),
                )
                .entity(
                    PriorActorSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        ActionRecordIdentityField::reference(),
                        encoded_string("selected-prior"),
                    ),
                ),
        ))
    }
}

fn encoded_string(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned())
        .expect("publication request fixture value must encode")
}
