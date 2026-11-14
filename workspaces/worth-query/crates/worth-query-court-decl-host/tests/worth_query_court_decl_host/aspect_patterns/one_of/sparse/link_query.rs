//! Ordinary current queries independently observe the optional native link.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Row {
    pub targets: Vec<String>,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity: "native-record-patterns.sparse.link-parameters.v1"});
worth_query_structured_value_binding!(pub ResultBinding for Row {identity: "native-record-patterns.sparse.links.v1"});
worth_query_application_query!(pub Query for Schema, identity "native-record-patterns.sparse.link-query.v1", parameters ParametersBinding, result ResultBinding, scope Record => "Record", name "record-links");
pub struct Links;
pub struct TargetKey;
worth_query_portable_type!(Links => "native-record-patterns.sparse.links-slot.v1");
worth_query_portable_type!(TargetKey => "native-record-patterns.sparse.target-key-slot.v1");
fn target_key() -> ApplicationQueryResultFieldRef<Query, TargetKey, Schema, Key> {
    ApplicationQueryResultFieldRef::new("key", Key::at("Key"))
}
fn links() -> ApplicationQueryResultRelationRef<
    Query,
    Links,
    Schema,
    Companion,
    Record,
    Record,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("links", Companion::reference())
}
pub fn definition() -> ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, Row, Record> {
    let child =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Record, Row, ResultBinding>::new(
            Record::reference(),
        )
        .field(target_key());
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Record, Row, ResultBinding>::new(
            Record::reference(),
        )
        .relation(links(), child)
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
            targets: row
                .many(links())?
                .iter()
                .map(|target| target.field(target_key()))
                .collect::<Result<_, _>>()?,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read {
    pub key: String,
}
worth_query_structured_value_binding!(pub InputBinding for Read {identity: "native-record-patterns.sparse.link-input.v1"});
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
    const IDENTITY: &'static str = "native-record-patterns.sparse.link-binding.v1";
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
