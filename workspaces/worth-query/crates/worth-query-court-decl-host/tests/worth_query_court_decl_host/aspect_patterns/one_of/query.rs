use super::*;
#[derive(Clone, Debug, PartialEq, Eq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Row {
    pub class: u64,
    pub first: Option<u64>,
    pub second: Option<u64>,
    pub required: Option<u64>,
    pub third: Option<u64>,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "native-record-patterns.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "native-record-patterns.row.v1" });
worth_query_application_query!(pub Query for Schema, identity "native-record-patterns.read.v1", parameters ParametersBinding, result ResultBinding, scope Record => "Record", name "record");
pub struct FirstValueSlot;
worth_query_portable_type!(FirstValueSlot => "native-record-patterns.first-slot.v1");
fn first() -> ApplicationQueryOptionalAspectFieldRef<Query, FirstValueSlot, Schema, FirstValue> {
    ApplicationQueryOptionalAspectFieldRef::new("first", FirstValue::at("FirstValue"))
}
pub struct SecondValueSlot;
worth_query_portable_type!(SecondValueSlot => "native-record-patterns.second-slot.v1");
fn second() -> ApplicationQueryOptionalAspectFieldRef<Query, SecondValueSlot, Schema, SecondValue> {
    ApplicationQueryOptionalAspectFieldRef::new("second", SecondValue::at("SecondValue"))
}
pub struct SecondRequiredSlot;
worth_query_portable_type!(SecondRequiredSlot => "native-record-patterns.required-slot.v1");
fn required(
) -> ApplicationQueryOptionalAspectFieldRef<Query, SecondRequiredSlot, Schema, SecondRequired> {
    ApplicationQueryOptionalAspectFieldRef::new("required", SecondRequired::at("SecondRequired"))
}
pub struct ThirdValueSlot;
worth_query_portable_type!(ThirdValueSlot => "native-record-patterns.third-slot.v1");
fn third() -> ApplicationQueryOptionalAspectFieldRef<Query, ThirdValueSlot, Schema, ThirdValue> {
    ApplicationQueryOptionalAspectFieldRef::new("third", ThirdValue::at("ThirdValue"))
}
pub fn definition() -> ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, Row, Record> {
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Record, Row, ResultBinding>::new(
            Record::reference(),
        )
        .optional_aspect_field(first())
        .optional_aspect_field(second())
        .optional_aspect_field(required())
        .optional_aspect_field(third())
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
        let first = row.optional_aspect_field(first())?;
        let second = row.optional_aspect_field(second())?;
        let required = row.optional_aspect_field(required())?;
        let third = row.optional_aspect_field(third())?;
        let class = match (first, second, required, third) {
            (Some(_), None, None, None) => 1,
            (None, Some(_), Some(_), None) => 2,
            (None, None, None, Some(_)) => 3,
            _ => {
                return Err(WorthQueryApplicationProjectionDenial::reject(
                    "record case corruption",
                ))
            }
        };
        Ok(Self {
            class,
            first,
            second,
            required,
            third,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read {
    pub key: String,
}
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "native-record-patterns.read-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<Key>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "native-record-patterns.read-binding.v1";
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
    type Binding = Binding;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope {
        Scope::new(Key::at("Key"), self.key)
    }
}
