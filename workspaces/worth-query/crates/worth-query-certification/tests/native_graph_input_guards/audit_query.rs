//! Independent governed readback of audit publication.
use super::schema::*;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_portable_type, worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
#[derive(Clone)]
pub struct Read(pub String);
#[derive(Clone, Debug)]
pub struct Value(pub u64);
worth_query_portable_type!(Value => "worth.query.path-input.audit_query.value.v1");
worth_query_structured_value_binding!(pub Input for Read { identity: "worth.query.path-input.audit_query.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Value { identity: "worth.query.path-input.audit_query.value.v1" });
worth_query_structured_value_binding!(pub Parameters for () { identity: "worth.query.path-input.audit_query.parameters.v1" });
pub struct ValueSlot;
worth_query_portable_type!(ValueSlot => "worth.query.path-input.audit_query.slot.v1");
worth_query_application_query!(pub Query for Schema, identity "AuditQuery", parameters Parameters,
    result ResultBinding, scope Audit => "Audit", name "audit_query");
fn value() -> ApplicationQueryResultFieldRef<
    Query,
    ValueSlot,
    Schema,
    Audit,
    AuditFacts,
    AuditValue,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", AuditValue::reference())
}
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, (), Value, Audit> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Audit::reference())
        .scope(Audit::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Audit, Value, ResultBinding>::new(
                Audit::reference(),
            )
            .field(value())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<Schema, Query> for Value {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(row.field(value())?))
    }
}
fn identity(read: Read) -> String {
    read.0
}
worth_query_query_binding!(pub ReadBinding for Read, schema Schema,
    identity "worth.query.path-input.audit_query.binding.v1", input Input, query Query,
    parameters Parameters => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Audit, AuditFacts, AuditIdentity, String, ReadOnly, NoApplicationUnit, field AuditIdentity::reference(), value identity, limits results 1);
