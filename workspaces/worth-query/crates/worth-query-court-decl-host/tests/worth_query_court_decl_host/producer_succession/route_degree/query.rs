//! Parameter coordinates distinguish routes while the entity scope stays fixed.
use super::*;
#[derive(Clone, Debug, serde::Serialize)]
pub struct ReadRoute {
    pub key: String,
    pub partition: u64,
}
worth_query_structured_value_binding!(pub RouteInput for ReadRoute { identity:"dispatch.parameter.input" });
worth_query_application_query!(pub RouteQuery for Schema, identity "DispatchParameterRead", parameters Parameters, result RowBinding, scope Node => "Node", name "dispatch_parameter_read");
pub struct PartitionSlot;
pub fn partition(
) -> ApplicationQueryParameterRef<RouteQuery, PartitionSlot, U64ApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("partition")
}
macro_rules! result_field {
    ($slot:ident, $fn:ident, $field:ident, $value:ty, $write:ty, $name:literal) => {
        pub struct $slot;
        worth_query_portable_type!($slot => $name);
        pub fn $fn() -> ApplicationQueryResultFieldRef<RouteQuery, $slot, Schema, $field> {
            ApplicationQueryResultFieldRef::new(stringify!($field), $field::at(stringify!($field)))
        }
    };
}
result_field!(
    KeySlot,
    key_field,
    Key,
    String,
    ReadOnly,
    "dispatch.parameter.key"
);
result_field!(
    ParentSlot,
    parent_field,
    ParentName,
    String,
    ReadOnly,
    "dispatch.parameter.parent"
);
result_field!(
    ValueSlot,
    value_field,
    Value,
    u64,
    ReadWrite,
    "dispatch.parameter.value"
);
result_field!(
    WidthSlot,
    width_field,
    Width,
    u64,
    ReadWrite,
    "dispatch.parameter.width"
);
pub fn definition() -> ApplicationQueryDefinitionBuilder<Schema, RouteQuery, (), Row, Node> {
    ApplicationQueryDefinitionBuilder::declare(RouteQuery::reference())
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, RouteQuery, Node, Row, RowBinding>::new(
                Node::reference(),
            )
            .field(key_field())
            .field(parent_field())
            .field(value_field())
            .field(width_field())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(partition())
}
impl WorthQueryApplicationProjection<Schema, RouteQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, RouteQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            native_id: row.entity_id(),
            key: row.field(key_field())?,
            generated_by: row.field(parent_field())?,
            value: row.field(value_field())?,
            width: row.field(width_field())?,
        })
    }
}
fn scope(input: ReadRoute) -> String {
    input.key
}
worth_query_query_binding!(pub Binding for ReadRoute, schema Schema, identity "dispatch.parameter.read", input RouteInput,
    query RouteQuery, parameters Parameters => |input:&ReadRoute| ApplicationQueryParameterSet::new().bind(partition(),input.partition).unwrap(), result RowBinding,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key, field Key::at("Key"), value scope, limits results 1);
