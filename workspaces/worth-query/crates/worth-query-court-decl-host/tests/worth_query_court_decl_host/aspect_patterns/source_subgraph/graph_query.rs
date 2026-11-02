pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "source-subgraph.parameters.v1" });
pub struct KeySlot;
pub struct ValueSlot;
pub struct MemberSlot;
pub struct MemberKeySlot;
pub struct MemberValueSlot;
worth_query_portable_type!(MemberKeySlot => "source-subgraph.member-key-slot.v1");
worth_query_portable_type!(MemberValueSlot => "source-subgraph.member-value-slot.v1");
worth_query_portable_type!(KeySlot => "source-subgraph.key-slot.v1");
worth_query_portable_type!(ValueSlot => "source-subgraph.value-slot.v1");
worth_query_portable_type!(MemberSlot => "source-subgraph.member-slot.v1");
fn key() -> ApplicationQueryResultFieldRef<
    Query,
    KeySlot,
    Schema,
    Entity,
    Facts,
    Key,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", Key::reference())
}
fn value() -> ApplicationQueryResultFieldRef<
    Query,
    ValueSlot,
    Schema,
    Entity,
    Facts,
    Value,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", Value::reference())
}
fn member_key() -> ApplicationQueryResultFieldRef<
    Query,
    MemberKeySlot,
    Schema,
    Entity,
    Facts,
    Key,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("member_key", Key::reference())
}
fn member_value() -> ApplicationQueryResultFieldRef<
    Query,
    MemberValueSlot,
    Schema,
    Entity,
    Facts,
    Value,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("member_value", Value::reference())
}
fn members() -> ApplicationQueryResultRelationRef<
    Query,
    MemberSlot,
    Schema,
    Relation,
    Entity,
    Entity,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("members", Relation::reference())
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let child =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Entity, GraphRow, RowBinding>::new(
            Entity::reference(),
        )
        .field(member_key())
        .field(member_value());
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Entity, GraphRow, RowBinding>::new(
            Entity::reference(),
        )
        .field(key())
        .field(value())
        .relation(members(), child)
        .build();
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Entity::reference())
        .scope(Entity::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap();
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
impl WorthQueryApplicationProjection<Schema, Query> for GraphRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut children = row
            .many(members())?
            .iter()
            .map(|child| {
                Ok(MemberRow {
                    key: child.field(member_key())?,
                    value: child.field(member_value())?,
                })
            })
            .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
        children.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(Self {
            key: row.field(key())?,
            value: row.field(value())?,
            members: children,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "source-subgraph.read-input.v1" });
pub struct Binding;
type Scope =
    ApplicationQueryFieldScope<Schema, Entity, Facts, Key, String, ReadOnly, NoApplicationUnit>;
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
    const IDENTITY: &'static str = BINDING_ID;
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Entity,
        Facts,
        Key,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Key::reference()
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
        Scope::new(Key::reference(), self.0)
    }
}
