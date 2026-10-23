//! Native capability declaration for adopting the numeric model's program.
use super::schema::*;
use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
    worth_query_operation_reads,
};
use worth_query_host::facade::{
    declaration::{application_capability::*, application_program::*, application_schema::*},
    *,
};
mod contract;
mod seed;
pub use seed::seed;
worth_query_entity!(pub Grant for NumericSchema);
worth_query_aspect!(pub GrantFacts for NumericSchema, Grant; identity = AspectIdentity(0x91800104), revision = AspectContractRevision(1));
worth_query_field!(pub GrantIdentity for NumericSchema, Grant, GrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantAction for NumericSchema, Grant, GrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantPurpose for NumericSchema, Grant, GrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantStatus for NumericSchema, Grant, GrantFacts: String => StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub GrantScopeKey for NumericSchema, Grant, GrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantNotBefore for NumericSchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantNotAfter for NumericSchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantDepth for NumericSchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub GrantResource in NumericSchema, Grant => Number; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantee in NumericSchema, Principal => Grant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantor in NumericSchema, Principal => Grant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantParent in NumericSchema, Grant => Grant; integrity = same_context_unbounded_retain_dangling);
worth_query_capability!(pub AdoptCapability in NumericSchema);
worth_query_capability_context!(pub AdoptContext in NumericSchema);
worth_query_capability_provenance!(pub AdoptProvenance in NumericSchema);
#[derive(Clone, serde::Serialize)]
pub struct AdoptInput {
    pub source: String,
    pub target: String,
}
worth_query_portable_type!(AdoptInput => "numeric-correction.adopt-input.v1");
worth_query_structured_value_binding!(pub AdoptInputBinding for AdoptInput { identity: "numeric-correction.adopt-input.v1" });
worth_query_operation!(pub AdoptOperation for NumericSchema, input AdoptInputBinding);
worth_query_operation_reads!(AdoptOperation => [Key]);

pub struct AdoptBinding;
impl ApplicationProgramAdoptionBinding<NumericSchema> for AdoptBinding {
    const IDENTITY: &'static str = "numeric-correction.adopt-binding.v1";
    type Capability = AdoptCapability;
    type Input = AdoptInput;
    type InputBinding = AdoptInputBinding;
    type Operation = AdoptOperation;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        NumericSchema,
        ActorBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ActorBinding::reference()
    }
    fn input(context: ApplicationProgramAdoptionContext<'_>) -> AdoptInput {
        AdoptInput {
            source: context.source.to_string(),
            target: context.target.to_string(),
        }
    }
}
impl ApplicationCapabilityRequest<NumericSchema, AdoptCapability> for AdoptInput {
    type Scope = Number;
    type Context = AdoptContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<NumericSchema, Number, AdoptContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(Key::reference(), encoded("numeric")),
            encoded("adopt-program"),
            encoded("numeric-administration"),
            ApplicationCapabilityRequestContext::new(AdoptContext::reference()),
        ))
    }
}
fn encoded(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .entity(Grant::reference())
        .aspect(Grant::reference(), GrantFacts::reference())
        .field(Grant::reference(), GrantIdentity::reference())
        .field(Grant::reference(), GrantAction::reference())
        .field(Grant::reference(), GrantPurpose::reference())
        .field(Grant::reference(), GrantStatus::reference())
        .field(Grant::reference(), GrantScopeKey::reference())
        .field(Grant::reference(), GrantNotBefore::reference())
        .field(Grant::reference(), GrantNotAfter::reference())
        .field(Grant::reference(), GrantDepth::reference())
        .relation(
            GrantResource::reference(),
            Grant::reference(),
            Number::reference(),
        )
        .relation(
            GrantGrantee::reference(),
            Principal::reference(),
            Grant::reference(),
        )
        .relation(
            GrantGrantor::reference(),
            Principal::reference(),
            Grant::reference(),
        )
        .relation(
            GrantParent::reference(),
            Grant::reference(),
            Grant::reference(),
        )
        .capability_context(AdoptContext::reference())
        .capability_provenance(AdoptProvenance::reference())
        .operation(
            AdoptOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(AdoptOperation::reference(), 512)
        .operation_read_field(AdoptOperation::reference(), Key::reference())
        .capability(contract::contract())
}
