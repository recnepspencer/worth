//! Outgoing discovery has no unused incoming-parent observations.
use super::*;
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.component-dependents.v1",
    parameters query::ParametersBinding, result ResultBinding, scope Model => "Model", name "ComponentDependents");
#[derive(Clone, Debug)]
pub struct Dependents {
    pub children: Vec<String>,
}
worth_query_structured_value_binding!(pub ResultBinding for Dependents { identity:"owned-outputs.component-dependents-result.v1" });
pub struct Links;
pub struct Child;
pub struct Key;
worth_query_portable_type!(Links=>"owned-outputs.component-dependent-links");
worth_query_portable_type!(Child=>"owned-outputs.component-dependent-model");
worth_query_portable_type!(Key=>"owned-outputs.component-dependent-key");
fn links() -> ApplicationQueryResultRelationRef<
    Query,
    Links,
    Schema,
    UpstreamOf,
    Member,
    Model,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("dependents", UpstreamOf::reference())
}
fn child() -> ApplicationQueryResultRelationRef<
    Query,
    Child,
    Schema,
    ModelMember,
    Model,
    Member,
    ReverseResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("dependent", ModelMember::reference())
}
fn key() -> ApplicationQueryResultFieldRef<Query, Key, Schema, ModelKey> {
    ApplicationQueryResultFieldRef::new("key", ModelKey::at("ModelKey"))
}
impl WorthQueryApplicationProjection<Schema, Query> for Dependents {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            children: row
                .many(links())?
                .iter()
                .map(|row| row.one(child())?.field(key()))
                .collect::<Result<_, _>>()?,
        })
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let model = || {
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Dependents, ResultBinding>::new(
            Model::reference(),
        )
    };
    let member = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Member,
        Dependents,
        ResultBinding,
    >::new(Member::reference())
    .relation(child(), model().field(key()));
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(model().relation(links(), member).build())
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
worth_query_structured_value_binding!(pub Input for Read { identity:"owned-outputs.component-dependents-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<ModelKey>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = Input;
    type Query = Query;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.component-dependents-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ModelKey {
        ModelKey::at("ModelKey")
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
        Scope::new(ModelKey::at("ModelKey"), self.0)
    }
}
