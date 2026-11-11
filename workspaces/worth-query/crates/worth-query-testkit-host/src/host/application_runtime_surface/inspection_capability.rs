//! Ordinary declared grant for Host query and diagnostic access.
#[path = "inspection_capability/records.rs"]
mod storage_model;
use super::{Account, AccountNumber, HostIdentitySchema, IdentityModel, Principal};
pub use storage_model::*;
use worth_query_decl::facade::{
    application_capability::*,
    application_schema::{
        ApplicationEncodedScalarValue, BoolApplicationValueBinding, StringApplicationValueBinding,
        U64ApplicationValueBinding,
    },
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
    worth_query_operation, worth_query_operation_reads, worth_query_structured_value_binding,
};

worth_query_capability_context!(pub InspectionContext in HostIdentitySchema);
worth_query_capability_provenance!(pub InspectionProvenance in HostIdentitySchema);
worth_query_capability!(pub InspectAccountCapability in HostIdentitySchema);
worth_query_structured_value_binding!(pub InspectAccountInputBinding for String {
    identity: "worth.query.host.inspection.capability-input.v1"
});
worth_query_operation!(pub InspectAccountOperation for HostIdentitySchema, input InspectAccountInputBinding);

pub(super) fn text(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
}
impl ApplicationCapabilityRequest<HostIdentitySchema, InspectAccountCapability> for String {
    type Scope = Account;
    type Context = InspectionContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<HostIdentitySchema, Account, InspectionContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                AccountNumber::at("AccountNumber"),
                text(self),
            )
            .unwrap(),
            text("inspect"),
            text("audit"),
            ApplicationCapabilityRequestContext::new(InspectionContext::reference()),
        )
        .field(text("number")))
    }
}
#[path = "inspection_capability/contract.rs"]
mod contract;

#[path = "inspection_capability/seed.rs"]
mod seed;
pub(super) use seed::bind;
