//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct PublicationRecords;
impl fragment::Fragment for PublicationRecords {
    const NAME: &'static str = "publication_records";
    type Bindings = PublicationRecordsBindings;
    type Members = (ExternalMapping, Principal, Resource, ActionRecord);
}
pub struct PublicationRecordsBindings;
impl fragment::BindingSet for PublicationRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl
    fragment::Scalar<
        worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
    > for PublicationRecordsBindings
{
    type Binding =
        worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl
    fragment::Scalar<
        worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus,
    > for PublicationRecordsBindings
{
    type Binding =
        worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for PublicationRecordsBindings {
    type Binding = worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for PublicationRecordsBindings {
    type Binding =
        worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}

#[path = "records/entity_members.rs"]
mod entity_members;
pub use entity_members::{ActionRecord, CapabilityGrant, ExternalMapping, Principal, Resource};
#[path = "records/value_rows.rs"]
mod value_rows;
pub use value_rows::{
    ActionRecordFacts, ActionRecordFactsValue, ExternalIdentity, ExternalIdentityValue, GrantFacts,
    GrantFactsValue, PrincipalFacts, PrincipalFactsValue, ResourceFacts, ResourceFactsValue,
};
#[path = "records/selections.rs"]
mod selections;
pub use selections::{
    ActionRecordIdentityField, ExternalIdentityField, GrantActionField, GrantDelegationLimitField,
    GrantIdentityField, GrantNotAfterField, GrantNotBeforeField, GrantPurposeField,
    GrantStatusField, GrantWorkflowField, MappingStatusField, PrincipalIdentityField,
    ResourceIdentityField, ResourceLabelField, ResourceWorkflowField,
};
#[path = "records/relations.rs"]
mod relations;
pub use relations::{
    ActionResource, ConflictingActor, ExplicitDeny, GrantBindings, GrantGrantee, GrantGrantor,
    GrantKey, GrantParent, GrantRecords, GrantResource, MappingTarget, PriorActor, RequestActor,
    ResourceOwner,
};
