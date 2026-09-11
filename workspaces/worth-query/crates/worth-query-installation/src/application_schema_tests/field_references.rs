//! Schema-owned marker family for the installation schema fixtures.

use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEntityMarkerIdentity,
    ApplicationFieldMarkerIdentity, ApplicationFieldPresence, DeclaredApplicationFieldValue,
    OperationCreates, OperationExpectsFact, OperationReads, U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

use super::{TestOperation, TestSchema};

pub(super) struct FixtureEntity<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct FixtureIdentityAspect<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct FixtureExternalIdentityField<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct FixtureMappingStatusField<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct FixturePrincipalIdentityField<Schema>(PhantomData<fn() -> Schema>);

pub(super) type TestEntity = FixtureEntity<TestSchema>;

impl<Schema> ApplicationEntityMarkerIdentity<Schema> for FixtureEntity<Schema> {
    const IDENTIFIER: &'static str = "TestEntity";
}

impl<Schema> ApplicationAspectMarkerIdentity<Schema, FixtureEntity<Schema>>
    for FixtureIdentityAspect<Schema>
{
    const IDENTIFIER: &'static str = "IdentityAspect";
    const ASPECT_IDENTITY: worth_query_declaration::facade::application_schema::AspectIdentity =
        worth_query_declaration::facade::application_schema::AspectIdentity(0x9161200c);
    const CONTRACT_REVISION:
        worth_query_declaration::facade::application_schema::AspectContractRevision =
        worth_query_declaration::facade::application_schema::AspectContractRevision(1);
}

macro_rules! field_marker_identity {
    ($marker:ident, $identifier:literal) => {
        impl<Schema>
            ApplicationFieldMarkerIdentity<
                Schema,
                FixtureEntity<Schema>,
                FixtureIdentityAspect<Schema>,
            > for $marker<Schema>
        {
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

field_marker_identity!(FixtureExternalIdentityField, "ExternalIdentityField");
field_marker_identity!(FixtureMappingStatusField, "MappingStatusField");
field_marker_identity!(FixturePrincipalIdentityField, "PrincipalIdentityField");

macro_rules! required_field {
    ($field:ident, $value:ty, $binding:ty) => {
        impl<Schema> DeclaredApplicationFieldValue for $field<Schema> {
            type Value = $value;
            type Binding = $binding;
            const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
        }
    };
}

required_field!(
    FixtureExternalIdentityField,
    WorthQueryExternalPrincipalIdentity,
    WorthQueryExternalPrincipalIdentityBinding
);
required_field!(
    FixtureMappingStatusField,
    WorthQueryPrincipalMappingStatus,
    WorthQueryPrincipalMappingStatusBinding
);
required_field!(
    FixturePrincipalIdentityField,
    u64,
    U64ApplicationValueBinding
);

impl<Schema> OperationCreates<TestOperation<Schema>> for FixtureEntity<Schema> {}
impl<Schema> OperationReads<TestOperation<Schema>> for FixturePrincipalIdentityField<Schema> {}
impl<Schema> OperationExpectsFact<TestOperation<Schema>> for FixturePrincipalIdentityField<Schema> {}
