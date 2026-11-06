//! Ordinary independently admitted field disclosure for causal inspection.
use super::super::{adoption::*, schema::*};
use worth_query_decl::facade::{worth_query_capability, worth_query_operation_reads};
use worth_query_host::facade::{
    declaration::{application_capability::*, application_schema::*},
    primary_graph::*,
    *,
};
mod contract;

worth_query_capability!(pub Inspect in NumericSchema);
#[derive(Clone, serde::Serialize)]
pub struct Input {
    pub key: String,
}
impl Input {
    pub fn at(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}
worth_query_portable_type!(Input => "numeric-correction.inspect-input.v2");
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "numeric-correction.inspect-input.v2" });
worth_query_operation!(pub InspectCausalEvidence for NumericSchema, input InputBinding);

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
            ApplicationCapabilityEntitySelector::new(Key::at("Key"), text(&self.key)).unwrap(),
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
        .operation(
            InspectCausalEvidence::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(InspectCausalEvidence::reference(), 512)
        .operation_read_field(InspectCausalEvidence::reference(), Key::at("Key"))
        .capability(contract::definition())
}
pub fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>) {
    let grant =
        || WorthQueryApplicationEntityKey::<NumericSchema, Grant>::new("inspection-grant").unwrap();
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(Grant::reference(), grant())
                .field(
                    GrantIdentity::at("GrantIdentity"),
                    "inspection-grant".to_owned(),
                )
                .field(GrantAction::at("GrantAction"), "inspect".to_owned())
                .field(GrantPurpose::at("GrantPurpose"), "audit".to_owned())
                .field(Disclosure::at("Disclosure"), "calculation".to_owned())
                .field(GrantStatus::at("GrantStatus"), "active".to_owned())
                .field(GrantScopeKey::at("GrantScopeKey"), "numeric".to_owned())
                .field(GrantNotBefore::at("GrantNotBefore"), 0_u64)
                .field(GrantNotAfter::at("GrantNotAfter"), u64::MAX)
                .field(GrantDepth::at("GrantDepth"), 0_u64),
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

pub use crate::facade::certification::application_graph::correction::model::schema::storage_model::Disclosure;
