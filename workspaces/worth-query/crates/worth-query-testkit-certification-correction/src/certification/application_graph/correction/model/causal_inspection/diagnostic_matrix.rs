//! One installed query shape with independently declared explanation policies.
use super::super::{authored_identity_model::AuthoredReference, schema::*};
use super::{capability, independent};
use worth_query_decl::facade::worth_query_query_binding;
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
pub mod journeys;
mod shape;
use shape::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub operand: Option<u64>,
    pub references: Vec<(u64, u64)>,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "diagnostic-matrix.row.v1" });
worth_query_structured_value_binding!(pub ChildBinding for () { identity: "diagnostic-matrix.child.v1" });
worth_query_structured_value_binding!(pub ParametersBinding for () { identity: "diagnostic-matrix.parameters.v1" });
pub struct EnabledParameter;
pub fn enabled<Q>() -> ApplicationQueryParameterRef<Q, EnabledParameter, BoolApplicationValueBinding>
{
    ApplicationQueryParameterRef::from_query_identifier("enabled")
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum Policy {
    Complete,
    Internal,
    Relation,
    Field,
}

macro_rules! binding {
    ($query:ident, $read:ident, $input:ident, $binding:ident, $identity:literal, $binding_identity:literal, $name:literal) => {
        worth_query_application_query!(pub $query for NumericSchema, identity $identity,
            parameters ParametersBinding, result ResultBinding, scope Number => "Number", name $name);
        #[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
        pub struct $read { pub key: String, pub enabled: bool }
        impl $read { fn into_key(self) -> String { self.key } }
        worth_query_structured_value_binding!(pub $input for $read { identity: concat!($identity, ".input") });
        worth_query_query_binding!(pub $binding for $read, schema NumericSchema, identity $binding_identity,
            input $input, query $query, parameters ParametersBinding => |input: &$read|
                ApplicationQueryParameterSet::new().bind(enabled::<$query>(), input.enabled).unwrap(), result ResultBinding,
            principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
            scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
            field Key::reference(), value $read::into_key, limits results 1);
        impl WorthQueryApplicationProjection<NumericSchema, $query> for Row {
            fn project(row: &WorthQueryApplicationProjectionRow<'_, NumericSchema, $query>)
                -> Result<Self, WorthQueryApplicationProjectionDenial> {
                Ok(Self { operand: row.optional_field(operand())?, references: row.many(references())?
                    .iter().map(|child| Ok((child.field(ordering())?, child.field(calculated())?))).collect::<Result<_, WorthQueryApplicationProjectionDenial>>()? })
            }
        }
    };
}
binding!(
    Complete,
    Read,
    Input,
    Binding,
    "diagnostic-matrix.complete.v1",
    "diagnostic-matrix.complete-binding.v1",
    "matrix_complete"
);
binding!(
    Internal,
    InternalRead,
    InternalInput,
    InternalBinding,
    "diagnostic-matrix.internal.v1",
    "diagnostic-matrix.internal-binding.v1",
    "matrix_internal"
);
binding!(
    Relation,
    RelationRead,
    RelationInput,
    RelationBinding,
    "diagnostic-matrix.relation.v1",
    "diagnostic-matrix.relation-binding.v1",
    "matrix_relation"
);
binding!(
    Field,
    FieldRead,
    FieldInput,
    FieldBinding,
    "diagnostic-matrix.field.v1",
    "diagnostic-matrix.field-binding.v1",
    "matrix_field"
);

fn definition<Q>(
    reference: ApplicationQueryReference<NumericSchema, Q, (), Row, Number>,
    policy: Policy,
) -> ApplicationQueryDefinition<NumericSchema, Q, (), Row, Number>
where
    Q: ApplicationQueryMarkerIdentity<NumericSchema> + 'static,
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = Row>,
{
    let child =
        ApplicationQueryResultShapeBuilder::<NumericSchema, Q, Number, (), ChildBinding>::new(
            Number::reference(),
        )
        .field(calculated())
        .field(ordering());
    let shape =
        ApplicationQueryResultShapeBuilder::<NumericSchema, Q, Number, Row, Q::ResultBinding>::new(
            Number::reference(),
        )
        .optional_field(operand())
        .relation(references(), child)
        .build();
    let influence = |denied: bool| {
        ApplicationQueryInfluenceContract::permit(
            ApplicationQueryObservableInfluence::ALL
                .into_iter()
                .filter(move |surface| {
                    !denied || *surface != ApplicationQueryObservableInfluence::Explanation
                }),
        )
    };
    let disclosure =
        ApplicationQueryDisclosureContract::governed_by("matrix", capability::Inspect::reference())
            .use_field_by(
                ReadEnabled::reference(),
                capability::text("calculation"),
                influence(policy == Policy::Internal),
            )
            .use_field_by(
                Primary::reference(),
                capability::text("calculation"),
                influence(false),
            )
            .disclose_optional_field_by(
                operand::<Q>(),
                capability::text("calculation"),
                influence(policy == Policy::Field),
            )
            .disclose_field_by(
                calculated::<Q>(),
                capability::text("calculation"),
                influence(false),
            )
            .disclose_field_by(
                ordering::<Q>(),
                capability::text("calculation"),
                influence(false),
            )
            .disclose_relation_by(
                references::<Q>(),
                capability::text("calculation"),
                influence(policy == Policy::Relation),
            );
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Number::reference())
        .scope(Number::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(enabled())
        .where_equal(ReadEnabled::reference(), enabled())
        .order_by(ordering(), ApplicationQueryOrderingDirection::Ascending)
        .build()
        .unwrap()
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .application_query(definition(Complete::reference(), Policy::Complete))
        .application_query_binding::<Binding>()
        .application_query_inspection(independent::metadata::<Binding>("matrix-complete-metadata"))
        .application_query(definition(Internal::reference(), Policy::Internal))
        .application_query_binding::<InternalBinding>()
        .application_query_inspection(independent::metadata::<InternalBinding>(
            "matrix-internal-metadata",
        ))
        .application_query(definition(Relation::reference(), Policy::Relation))
        .application_query_binding::<RelationBinding>()
        .application_query_inspection(independent::metadata::<RelationBinding>(
            "matrix-relation-metadata",
        ))
        .application_query(definition(Field::reference(), Policy::Field))
        .application_query_binding::<FieldBinding>()
        .application_query_inspection(independent::metadata::<FieldBinding>(
            "matrix-field-metadata",
        ))
}
