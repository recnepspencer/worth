//! The join source reads both real parent outputs through installed relations.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.joined-query.v1",
    parameters ParametersBinding, result RowBinding, scope FirstOutput => "FirstOutput", name "JoinedSource");
pub struct KeySlot;
pub struct ChildSlot;
pub struct ModelSlot;
pub struct SecondSlot;
pub struct ChildValueSlot;
pub struct SecondValueSlot;
worth_query_portable_type!(KeySlot => "owned-outputs.joined-key.v1");
worth_query_portable_type!(ChildSlot => "owned-outputs.joined-child.v1");
worth_query_portable_type!(ModelSlot => "owned-outputs.joined-model.v1");
worth_query_portable_type!(SecondSlot => "owned-outputs.joined-second.v1");
worth_query_portable_type!(ChildValueSlot => "owned-outputs.joined-child-value.v1");
worth_query_portable_type!(SecondValueSlot => "owned-outputs.joined-second-value.v1");
fn key() -> ApplicationQueryResultFieldRef<Query, KeySlot, Schema, FirstKey> {
    ApplicationQueryResultFieldRef::new("key", FirstKey::at("FirstKey"))
}
fn children() -> ApplicationQueryResultRelationRef<
    Query,
    ChildSlot,
    Schema,
    ChildOfFirst,
    FirstOutput,
    ChildOutput,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("children", ChildOfFirst::reference())
}
fn model() -> ApplicationQueryResultRelationRef<
    Query,
    ModelSlot,
    Schema,
    FirstOf,
    FirstOutput,
    Model,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("model", FirstOf::reference())
}
fn seconds() -> ApplicationQueryResultRelationRef<
    Query,
    SecondSlot,
    Schema,
    SecondOf,
    SecondOutput,
    Model,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("seconds", SecondOf::reference())
}
fn child_value() -> ApplicationQueryResultFieldRef<Query, ChildValueSlot, Schema, ChildValue> {
    ApplicationQueryResultFieldRef::new("child-value", ChildValue::at("ChildValue"))
}
fn second_value() -> ApplicationQueryResultFieldRef<Query, SecondValueSlot, Schema, SecondValue> {
    ApplicationQueryResultFieldRef::new("second-value", SecondValue::at("SecondValue"))
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let child =
        ApplicationQueryResultShapeBuilder::<Schema, Query, ChildOutput, Row, RowBinding>::new(
            ChildOutput::reference(),
        )
        .field(child_value());
    let second =
        ApplicationQueryResultShapeBuilder::<Schema, Query, SecondOutput, Row, RowBinding>::new(
            SecondOutput::reference(),
        )
        .field(second_value());
    let owner = ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Row, RowBinding>::new(
        Model::reference(),
    )
    .relation(seconds(), second);
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, FirstOutput, Row, RowBinding>::new(
            FirstOutput::reference(),
        )
        .field(key())
        .relation(children(), child)
        .relation(model(), owner)
        .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(FirstOutput::reference())
        .scope(FirstOutput::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public();
    schema
        .application_query(definition)
        .application_query_binding::<Binding>()
        .application_query_inspection(
            ApplicationQueryInspectionDefinition::<Schema, Binding>::new(
                ApplicationQueryDisclosureContract::public(),
            )
            .unwrap(),
        )
}
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let left = row
            .many(children())?
            .iter()
            .map(|child| child.field(child_value()))
            .collect::<Result<Vec<u64>, _>>()?;
        let right = row
            .one(model())?
            .many(seconds())?
            .iter()
            .map(|second| second.field(second_value()))
            .collect::<Result<Vec<u64>, _>>()?;
        Ok(Self {
            native_id: row.entity_id(),
            key: row.field(key())?,
            value: left.into_iter().sum::<u64>() + right.into_iter().sum::<u64>(),
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "owned-outputs.joined-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<FirstKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = ReadInput;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.joined-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> FirstKey {
        FirstKey::at("FirstKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Actor,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Actor::reference()
    }
}
impl ApplicationQueryIntent<Schema> for Read {
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(FirstKey::at("FirstKey"), self.0)
    }
}
