use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Row {
    pub sink: u64,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "class-read-cost.parameters.v1" });
worth_query_structured_value_binding!(pub RowBinding for Row { identity: "class-read-cost.row.v1" });
worth_query_application_query!(pub Query for Schema, identity "class-read-cost.read.v1", parameters ParametersBinding, result RowBinding, scope Record => "Record", name "record");
pub struct SinkSlot;
worth_query_portable_type!(SinkSlot => "class-read-cost.sink-slot.v1");
fn sink() -> ApplicationQueryResultFieldRef<Query, SinkSlot, Schema, Sink> {
    ApplicationQueryResultFieldRef::new("sink", Sink::at("Sink"))
}
pub fn definition() -> ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, Row, Record> {
    let shape = ApplicationQueryResultShapeBuilder::<Schema, Query, Record, Row, RowBinding>::new(
        Record::reference(),
    )
    .field(sink())
    .build();
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Record::reference())
        .scope(Record::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            sink: row.field(sink())?,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read;
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "class-read-cost.read-input.v1" });
pub struct ReadBinding;
type Scope = ApplicationQueryFieldScope<Key>;
impl ApplicationQueryBinding<Schema> for ReadBinding {
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
    const IDENTITY: &'static str = "class-read-cost.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> Key {
        Key::at("Key")
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
    type Binding = ReadBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(Key::at("Key"), "record".into())
    }
}
