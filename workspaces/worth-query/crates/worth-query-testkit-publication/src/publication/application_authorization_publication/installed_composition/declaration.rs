#[path = "declaration/records.rs"]
mod storage_model;
pub use storage_model::*;
pub use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRequest,
    ApplicationCapabilityRequestContext, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};
pub use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, StringApplicationValueBinding,
};
pub use worth_query_declaration::{
    worth_query_capability, worth_query_capability_context,
    worth_query_capability_context_entity_slot, worth_query_capability_provenance,
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_principal_binding,
};

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct PublicationAuthorizationSchema;
impl Fragment for PublicationAuthorizationSchema {
    const NAME: &'static str = "PublicationAuthorizationSchema";
    type Bindings = NoBindings;
    type Members = (PublicationRecords, GrantRecords, Contracts);
}
impl ApplicationSchema for PublicationAuthorizationSchema {
    const OWNER: &'static str = "publication_authorization_proof";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl PublicationAuthorizationSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<PublicationAuthorizationSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<PublicationAuthorizationSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<PublicationAuthorizationSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = schema
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
                .operation_projection_work_budget(PublicationOperation::reference(), 16)
                .operation_read_field(
                    PublicationOperation::reference(),
                    ResourceIdentityField::at("ResourceIdentityField"),
                )
                .operation_read_field(
                    PublicationOperation::reference(),
                    ActionRecordIdentityField::at("ActionRecordIdentityField"),
                )
                .operation_read_field(
                    PublicationOperation::reference(),
                    ResourceLabelField::at("ResourceLabelField"),
                )
                .operation_write(
                    PublicationOperation::reference(),
                    ResourceLabelField::at("ResourceLabelField"),
                );
            super::contract::install(schema)
        })
    }
}

pub struct PublicationIdentityBinding;
impl PublicationIdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        PublicationAuthorizationSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PublicationIdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

worth_query_capability_context!(pub PublicationRequestContext in PublicationAuthorizationSchema);
worth_query_capability_context_entity_slot!(pub RequestActorSlot in PublicationAuthorizationSchema, PublicationRequestContext => ActionRecord);
worth_query_capability_context_entity_slot!(pub PriorActorSlot in PublicationAuthorizationSchema, PublicationRequestContext => ActionRecord);
worth_query_capability_provenance!(pub PublicationCapabilityProvenance in PublicationAuthorizationSchema);
worth_query_capability!(pub PublicationCapability in PublicationAuthorizationSchema);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PublicationInput;
worth_query_declaration::worth_query_portable_type!(PublicationInput => "worth.query.test.publication.input.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub PublicationInputBinding for PublicationInput { identity: "worth.query.test.publication.input.v1" });

worth_query_operation!(pub PublicationOperation for PublicationAuthorizationSchema, input PublicationInputBinding);

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
                ResourceIdentityField::at("ResourceIdentityField"),
                encoded_string("resource-1"),
            )
            .unwrap(),
            encoded_string("inspect"),
            encoded_string("publication-proof"),
            ApplicationCapabilityRequestContext::new(PublicationRequestContext::reference())
                .entity(
                    RequestActorSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        ActionRecordIdentityField::at("ActionRecordIdentityField"),
                        encoded_string("selected-request"),
                    )
                    .unwrap(),
                )
                .entity(
                    PriorActorSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        ActionRecordIdentityField::at("ActionRecordIdentityField"),
                        encoded_string("selected-prior"),
                    )
                    .unwrap(),
                ),
        ))
    }
}

pub fn encoded_string(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned())
        .expect("publication request fixture value must encode")
}
