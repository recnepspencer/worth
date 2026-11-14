//! Declared absence and equality partition the same real Member adjacency.
use super::*;
mod courts;
pub mod expectation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Partition {
    pub ordinary: Vec<String>,
    pub matching: Vec<String>,
}
worth_query_structured_value_binding!(pub ResultBinding for Partition { identity: "owned-outputs.prerequisite-partition.result.v1" });
worth_query_structured_value_binding!(pub ChildBinding for String { identity: "owned-outputs.prerequisite-partition.child.v1" });
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.prerequisite-partition.query.v1",
    parameters query::ParametersBinding, result ResultBinding, scope Model => "Model", name "PrerequisitePartition");
pub struct Ordinary;
pub struct Matching;
pub struct OrdinaryKey;
pub struct MatchingKey;
pub struct Selected;
worth_query_portable_type!(Ordinary => "owned-outputs.prerequisite-partition.ordinary");
worth_query_portable_type!(Matching => "owned-outputs.prerequisite-partition.matching");
worth_query_portable_type!(OrdinaryKey => "owned-outputs.prerequisite-partition.ordinary-key");
worth_query_portable_type!(MatchingKey => "owned-outputs.prerequisite-partition.matching-key");
type Members<Slot> = ApplicationQueryResultRelationRef<
    Query,
    Slot,
    Schema,
    ModelMember,
    Model,
    Member,
    ForwardResultTraversal,
    ManyResults,
>;
fn ordinary() -> Members<Ordinary> {
    ApplicationQueryResultRelationRef::forward_many("ordinary", ModelMember::reference())
}
fn matching() -> Members<Matching> {
    ApplicationQueryResultRelationRef::forward_many("matching", ModelMember::reference())
}
fn ordinary_key() -> ApplicationQueryResultFieldRef<Query, OrdinaryKey, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("ordinary-key", MemberKey::at("MemberKey"))
}
fn matching_key() -> ApplicationQueryResultFieldRef<Query, MatchingKey, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("matching-key", MemberKey::at("MemberKey"))
}
fn selected() -> ApplicationQueryParameterRef<Query, Selected, U64ApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("selected")
}
fn parameters() -> ApplicationQueryParameterSet<Query> {
    ApplicationQueryParameterSet::new()
        .bind(selected(), 1)
        .unwrap()
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let ordinary_rows =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, String, ChildBinding>::new(
            Member::reference(),
        )
        .field(ordinary_key());
    let matching_rows =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, String, ChildBinding>::new(
            Member::reference(),
        )
        .field(matching_key());
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Model, Partition, ResultBinding>::new(
            Model::reference(),
        )
        .relation_where_absent(ordinary(), ordinary_rows, CalibrationReady::at("Ready"))
        .relation_where_equal(
            matching(),
            matching_rows,
            CalibrationReady::at("Ready"),
            selected(),
        )
        .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(selected());
    expectation::declare(
        schema
            .application_query(definition)
            .application_query_binding::<Binding>(),
    )
}
impl WorthQueryApplicationProjection<Schema, Query> for Partition {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            ordinary: row
                .many(ordinary())?
                .iter()
                .map(|row| row.field(ordinary_key()))
                .collect::<Result<_, _>>()?,
            matching: row
                .many(matching())?
                .iter()
                .map(|row| row.field(matching_key()))
                .collect::<Result<_, _>>()?,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub Input for Read { identity: "owned-outputs.prerequisite-partition.input.v1" });
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
    const IDENTITY: &'static str = "owned-outputs.prerequisite-partition.binding.v1";
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
        parameters()
    }
    fn into_scope(self) -> Scope {
        Scope::new(ModelKey::at("ModelKey"), self.0)
    }
}
pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for (key, calibration) in [
        ("partition:absent", None),
        ("partition:matching", Some(1)),
        ("partition:excluded", Some(2)),
    ] {
        let mut seed = WorthQueryApplicationEntitySeed::new(
            Member::reference(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        )
        .field(MemberKey::at("MemberKey"), key.to_owned())
        .field(MemberValue::at("MemberValue"), 0);
        if let Some(calibration) = calibration {
            seed = seed.field(CalibrationReady::at("Ready"), calibration);
        }
        graph.bind_entity(seed)?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ModelMember::reference(),
            format!("owns:{key}"),
            WorthQueryApplicationEntityKey::new("a").unwrap(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        ))?;
    }
    Ok(())
}
