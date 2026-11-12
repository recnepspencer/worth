//! Independently governed exact source field used by the real producer.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "predecessor.inspection-source.query.v1",
    parameters query::ParametersBinding, result query::RowBinding, scope Model => "Model", name "PredecessorInspectionSource");
pub struct KeySlot;
pub struct ValueSlot;
worth_query_portable_type!(KeySlot => "predecessor.inspection-source.key.v1");
worth_query_portable_type!(ValueSlot => "predecessor.inspection-source.value.v1");
fn key() -> ApplicationQueryResultFieldRef<Query, KeySlot, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("key", ModelKey::at("ModelKey"))
}
fn value() -> ApplicationQueryResultFieldRef<Query, ValueSlot, Schema, ModelValue> {
    ApplicationQueryResultFieldRef::new("value", ModelValue::at("ModelValue"))
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Model,
        query::Row,
        query::RowBinding,
    >::new(Model::reference())
    .field(key())
    .field(value())
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "predecessor-inspection-source",
        Inspect::reference(),
    )
    .disclose_field_by(
        key(),
        grant::text("a"),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        value(),
        grant::text("a"),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    schema
        .application_query(
            ApplicationQueryDefinitionBuilder::declare(Query::reference())
                .root(Model::reference())
                .scope(Model::reference())
                .result_shape(shape)
                .cardinality(ApplicationQueryCardinality::ExactlyOne)
                .disclosure(disclosure)
                .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
                .lanes(ApplicationQueryLaneEligibility::one_shot())
                .public(),
        )
        .application_query_binding::<Binding>()
        .application_query_inspection(metadata::<Binding>())
}
impl WorthQueryApplicationProjection<Schema, Query> for query::Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            native_id: row.entity_id(),
            key: row.field(key())?,
            value: row.field(value())?,
        })
    }
}

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "predecessor.inspection-source.input.v1" });
worth_query_query_binding!(pub Binding for Read, schema Schema, identity "predecessor.inspection-source.binding.v1",
    input InputBinding, query Query, parameters query::ParametersBinding => |_: &Read| ApplicationQueryParameterSet::new(), result query::RowBinding,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value into_key, limits results 1
);
fn into_key(read: Read) -> String {
    read.0
}
