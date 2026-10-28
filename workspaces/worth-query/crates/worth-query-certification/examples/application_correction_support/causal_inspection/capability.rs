//! Ordinary independently admitted field disclosure for causal inspection.
use super::super::{adoption::*, schema::*};
use worth_query_decl::facade::{worth_query_capability, worth_query_operation_reads};
use worth_query_host::facade::{
    declaration::{application_capability::*, application_schema::*},
    primary_graph::*,
    *,
};
mod contract;
worth_query_field!(pub Disclosure for NumericSchema, Grant, GrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_capability!(pub Inspect in NumericSchema);
#[derive(Clone, serde::Serialize)]
pub struct Input;
worth_query_portable_type!(Input => "numeric-correction.inspect-input.v1");
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "numeric-correction.inspect-input.v1" });
worth_query_operation!(pub InspectCausalEvidence for NumericSchema, input InputBinding);
worth_query_operation_reads!(InspectCausalEvidence => [Key]);
impl ApplicationCapabilityRequest<NumericSchema, Inspect> for Input {
    type Scope = Number;
    type Context = AdoptContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<NumericSchema, Number, AdoptContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(Key::reference(), text("numeric")),
            text("inspect"),
            text("audit"),
            ApplicationCapabilityRequestContext::new(AdoptContext::reference()),
        )
        .field(text("calculation")))
    }
}
pub fn text(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .field(Grant::reference(), Disclosure::reference())
        .operation(
            InspectCausalEvidence::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(InspectCausalEvidence::reference(), 512)
        .operation_read_field(InspectCausalEvidence::reference(), Key::reference())
        .capability(contract::definition())
}
pub fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>) {
    let grant =
        || WorthQueryApplicationEntityKey::<NumericSchema, Grant>::new("inspection-grant").unwrap();
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(Grant::reference(), grant())
                .field(GrantIdentity::reference(), "inspection-grant".to_owned())
                .field(GrantAction::reference(), "inspect".to_owned())
                .field(GrantPurpose::reference(), "audit".to_owned())
                .field(Disclosure::reference(), "calculation".to_owned())
                .field(GrantStatus::reference(), "active".to_owned())
                .field(GrantScopeKey::reference(), "numeric".to_owned())
                .field(GrantNotBefore::reference(), 0_u64)
                .field(GrantNotAfter::reference(), u64::MAX)
                .field(GrantDepth::reference(), 0_u64),
        )
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantResource::reference(),
            "inspection-resource",
            grant(),
            WorthQueryApplicationEntityKey::<NumericSchema, Number>::new("numeric").unwrap(),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantor::reference(),
            "inspection-grantor",
            WorthQueryApplicationEntityKey::<NumericSchema, Principal>::new("numeric-operator")
                .unwrap(),
            grant(),
        ))
        .unwrap();
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantee::reference(),
            "inspection-grantee",
            WorthQueryApplicationEntityKey::<NumericSchema, Principal>::new("numeric-operator")
                .unwrap(),
            grant(),
        ))
        .unwrap();
}
