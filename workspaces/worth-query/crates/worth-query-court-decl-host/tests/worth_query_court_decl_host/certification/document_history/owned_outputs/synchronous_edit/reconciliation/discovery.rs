//! Discovery selects recorded source endpoints; it never reads their derived children.
use super::*;
use query::{ParametersBinding, Row};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SourcesFound {
    pub sources: Vec<Row>,
}
worth_query_structured_value_binding!(pub ResultBinding for SourcesFound { identity: "reconciliation.discovery-result.v1" });
worth_query_application_query!(pub Query for Schema, identity "reconciliation.discovery.v1", parameters ParametersBinding, result ResultBinding, scope FirstOutput => "FirstOutput", name "ReconciliationSources");
pub struct ModelSlot;
pub struct Members;
pub struct Sources;
pub struct SourceKeySlot;
pub struct SourceValueSlot;
worth_query_portable_type!(ModelSlot => "reconciliation.discovery-model.v1");
worth_query_portable_type!(Members => "reconciliation.discovery-members.v1");
worth_query_portable_type!(Sources => "reconciliation.discovery-sources.v1");
worth_query_portable_type!(SourceKeySlot => "reconciliation.discovery-source-key.v1");
worth_query_portable_type!(SourceValueSlot => "reconciliation.discovery-source-value.v1");
fn members() -> ApplicationQueryResultRelationRef<
    Query,
    Members,
    Schema,
    FirstOf,
    FirstOutput,
    Model,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("members", FirstOf::reference())
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
fn sources() -> ApplicationQueryResultRelationRef<
    Query,
    Sources,
    Schema,
    ReconciledSourceOf,
    FirstOutput,
    SecondOutput,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("sources", ReconciledSourceOf::reference())
}
fn source_key() -> ApplicationQueryResultFieldRef<Query, SourceKeySlot, Schema, SecondKey> {
    ApplicationQueryResultFieldRef::new("source-key", SecondKey::at("SecondKey"))
}
fn source_value() -> ApplicationQueryResultFieldRef<Query, SourceValueSlot, Schema, SecondValue> {
    ApplicationQueryResultFieldRef::new("source-value", SecondValue::at("SecondValue"))
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let source = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        SecondOutput,
        SourcesFound,
        ResultBinding,
    >::new(SecondOutput::reference())
    .field(source_key())
    .field(source_value());
    let member = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        FirstOutput,
        SourcesFound,
        ResultBinding,
    >::new(FirstOutput::reference())
    .relation(sources(), source);
    let owner = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Model,
        SourcesFound,
        ResultBinding,
    >::new(Model::reference())
    .relation(members(), member);
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        FirstOutput,
        SourcesFound,
        ResultBinding,
    >::new(FirstOutput::reference())
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
impl WorthQueryApplicationProjection<Schema, Query> for SourcesFound {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut sources_found = Vec::new();
        for member in row.one(model())?.many(members())?.iter() {
            for source in member.many(sources())?.iter() {
                sources_found.push(Row {
                    native_id: source.entity_id(),
                    key: source.field(source_key())?,
                    value: source.field(source_value())?,
                });
            }
        }
        sources_found.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(Self {
            sources: sources_found,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub InputBinding for Read { identity: "reconciliation.discovery-input.v1" });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<FirstKey>;
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
    const IDENTITY: &'static str = "reconciliation.discovery-binding.v1";
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
        Scope::new(
            FirstKey::at("FirstKey"),
            format!("reconciled:root:{}", self.0),
        )
    }
}
