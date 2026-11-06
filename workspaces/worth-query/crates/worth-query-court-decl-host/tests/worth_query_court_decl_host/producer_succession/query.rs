use super::*;
#[derive(Clone, Debug)]
pub struct Read(pub String);
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Row {
    pub native_id: worth_relational::facade::identity::EntityId,
    pub key: String,
    pub generated_by: String,
    pub value: u64,
    pub width: u64,
}
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "dispatch.read.input" });
worth_query_structured_value_binding!(pub RowBinding for Row { identity: "dispatch.row" });
worth_query_structured_value_binding!(pub Parameters for () { identity: "dispatch.parameters" });
worth_query_application_query!(pub Query for Schema, identity "DispatchRead", parameters Parameters, result RowBinding, scope Node => "Node", name "dispatch_read");
macro_rules! result_field {
    ($slot:ident, $fn:ident, $field:ident, $value:ty, $write:ty, $name:literal) => {
        pub struct $slot;
        worth_query_portable_type!($slot => $name);
        pub fn $fn() -> ApplicationQueryResultFieldRef<Query, $slot, Schema, $field> {
            ApplicationQueryResultFieldRef::new(stringify!($field), $field::at(stringify!($field)))
        }
    };
}
result_field!(KeySlot, key_field, Key, String, ReadOnly, "dispatch.key");
result_field!(
    ParentSlot,
    parent_field,
    ParentName,
    String,
    ReadOnly,
    "dispatch.parent"
);
result_field!(
    ValueSlot,
    value_field,
    Value,
    u64,
    ReadWrite,
    "dispatch.value"
);
result_field!(
    WidthSlot,
    width_field,
    Width,
    u64,
    ReadWrite,
    "dispatch.width"
);
pub fn definition() -> ApplicationQueryDefinition<Schema, Query, (), Row, Node> {
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Node, Row, RowBinding>::new(
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
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
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
pub fn scope(read: Read) -> String {
    read.0
}
worth_query_query_binding!(pub ReadBinding for Read, schema Schema, identity "dispatch.read", input ReadInput,
    query Query, parameters Parameters => |_| ApplicationQueryParameterSet::new(), result RowBinding,
    principal PrincipalBinding, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key, field Key::at("Key"), value scope, limits results 1);
