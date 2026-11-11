//! Real produced fields observed through their declared model ownership relation.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "predecessor.inspection-output.query.v1",
    parameters query::ParametersBinding, result ResultBinding, scope Model => "Model", name "PredecessorInspectionOutput");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub values: Vec<u64>,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "predecessor.inspection-output.result.v1" });
pub struct OutputsSlot;
pub struct ValueSlot;
worth_query_portable_type!(OutputsSlot => "predecessor.inspection-output.outputs.v1");
worth_query_portable_type!(ValueSlot => "predecessor.inspection-output.value.v1");
fn outputs() -> ApplicationQueryResultRelationRef<
    Query,
    OutputsSlot,
    Schema,
    FirstOf,
    FirstOutput,
    Model,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("outputs", FirstOf::reference())
}
fn value() -> ApplicationQueryResultFieldRef<Query, ValueSlot, Schema, FirstValue> {
    ApplicationQueryResultFieldRef::new("value", FirstValue::at("FirstValue"))
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let child =
        ApplicationQueryResultShapeBuilder::<Schema, Query, FirstOutput, Row, ResultBinding>::new(
            FirstOutput::reference(),
        )
        .field(value());
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Row, ResultBinding>::new(
            Model::reference(),
        )
        .relation(outputs(), child)
        .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "predecessor-inspection-output",
        Inspect::reference(),
    )
    .disclose_field_by(
        value(),
        grant::text("a"),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_relation_by(
        outputs(),
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
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            values: row
                .many(outputs())?
                .iter()
                .map(|child| child.field(value()))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "predecessor.inspection-output.input.v1" });
worth_query_query_binding!(pub Binding for Read, schema Schema, identity "predecessor.inspection-output.binding.v1",
    input InputBinding, query Query, parameters query::ParametersBinding => |_: &Read| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ModelKey, field ModelKey::at("ModelKey"), value into_key, limits results 1
);
fn into_key(read: Read) -> String {
    read.0
}
