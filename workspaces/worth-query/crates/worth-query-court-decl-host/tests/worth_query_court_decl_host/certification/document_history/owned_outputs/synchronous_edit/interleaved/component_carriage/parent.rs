//! A selected FirstOutput's genuine source Model is its child's prerequisite.
use super::*;

worth_query_application_query!(pub Query for Schema, identity "owned-outputs.component-child-parent.v1",
    parameters query::ParametersBinding, result SourceBinding, scope FirstOutput => "FirstOutput", name "ComponentChildParent");
#[derive(Clone, Debug)]
pub struct Source;
#[derive(Clone, Debug)]
pub struct Parent {
    pub key: String,
}
worth_query_structured_value_binding!(pub SourceBinding for Source { identity:"owned-outputs.component-child-source.v1" });
worth_query_structured_value_binding!(pub ParentBinding for Parent { identity:"owned-outputs.component-child-parent-row.v1" });
pub struct Owner;
pub struct Key;
worth_query_portable_type!(Owner=>"owned-outputs.component-child-owner");
worth_query_portable_type!(Key=>"owned-outputs.component-child-owner-key");
pub type Selection = ApplicationQueryResultRelationRef<
    Query,
    Owner,
    Schema,
    FirstOf,
    FirstOutput,
    Model,
    ForwardResultTraversal,
    ExactlyOneResult,
>;
pub fn owner() -> Selection {
    ApplicationQueryResultRelationRef::forward_one("owner", FirstOf::reference())
}
fn key() -> ApplicationQueryResultFieldRef<Query, Key, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("key", ModelKey::at("ModelKey"))
}
impl WorthQueryApplicationProjection<Schema, Query> for Source {
    fn project(
        _: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self)
    }
}
impl WorthQueryApplicationProjection<Schema, Query> for Parent {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
        })
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let model =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Source, SourceBinding>::new(
            Model::reference(),
        )
        .field(key());
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        FirstOutput,
        Source,
        SourceBinding,
    >::new(FirstOutput::reference())
    .relation(owner(), model)
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
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub Input for Read { identity:"owned-outputs.component-child-parent-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<FirstKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = Input;
    type Query = Query;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = SourceBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.component-child-parent-binding.v1";
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
