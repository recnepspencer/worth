//! Ordinary declared grant for Host query and diagnostic access.
use super::{Account, AccountNumber, HostIdentitySchema, Principal};
use worth_query_decl::facade::{
    application_capability::*,
    application_schema::{
        ApplicationEncodedScalarValue, BoolApplicationValueBinding, StringApplicationValueBinding,
        U64ApplicationValueBinding,
    },
    worth_query_aspect, worth_query_capability, worth_query_capability_context,
    worth_query_capability_provenance, worth_query_entity, worth_query_field,
    worth_query_operation, worth_query_operation_reads, worth_query_relation,
    worth_query_structured_value_binding,
};
worth_query_entity!(pub InspectionGrant for HostIdentitySchema);
worth_query_aspect!(pub InspectionGrantFacts for HostIdentitySchema, InspectionGrant;
    identity = AspectIdentity(0x91811901), revision = AspectContractRevision(1),);
worth_query_field!(pub GrantIdentity for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantAction for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantPurpose for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantDisclosure for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub GrantActive for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    bool => BoolApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub GrantWorkflow for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub GrantNotBefore for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    u64 => U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub GrantNotAfter for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    u64 => U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub GrantDelegationLimit for HostIdentitySchema, InspectionGrant, InspectionGrantFacts:
    u64 => U64ApplicationValueBinding, read_write, no_equality);
worth_query_relation!(pub Grantor in HostIdentitySchema,
    Principal => InspectionGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Grantee in HostIdentitySchema,
    Principal => InspectionGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantResource in HostIdentitySchema,
    InspectionGrant => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantParent in HostIdentitySchema,
    InspectionGrant => InspectionGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_capability_context!(pub InspectionContext in HostIdentitySchema);
worth_query_capability_provenance!(pub InspectionProvenance in HostIdentitySchema);
worth_query_capability!(pub InspectAccountCapability in HostIdentitySchema);
worth_query_structured_value_binding!(pub InspectAccountInputBinding for String {
    identity: "worth.query.host.inspection.capability-input.v1"
});
worth_query_operation!(pub InspectAccountOperation for HostIdentitySchema, input InspectAccountInputBinding);
worth_query_operation_reads!(InspectAccountOperation => [AccountNumber]);

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
            ApplicationCapabilityEntitySelector::new(AccountNumber::reference(), text(self)),
            text("inspect"),
            text("audit"),
            ApplicationCapabilityRequestContext::new(InspectionContext::reference()),
        )
        .field(text("number")))
    }
}
#[path = "inspection_capability/contract.rs"]
mod contract;
pub(super) use contract::install;
#[path = "inspection_capability/seed.rs"]
mod seed;
pub(super) use seed::bind;
