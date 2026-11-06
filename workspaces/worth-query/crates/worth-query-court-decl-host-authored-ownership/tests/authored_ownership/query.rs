use super::*;
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Input {
    pub id: u64,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Row {
    pub id: u64,
    pub value: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "authored-ownership.input.v1" });
worth_query_structured_value_binding!(pub RowBinding for Row { identity: "authored-ownership.row.v1" });
worth_query_structured_value_binding!(pub UnitBinding for () { identity: "worth.rust.unit" });
worth_query_application_query!(pub Query for Schema: Scenario, identity "authored-ownership.read.v1", parameters UnitBinding, result RowBinding, scope Node<Schema> => "Node", name "owned_source");
pub struct Slot;
worth_query_portable_type!(Slot => "authored-ownership.id-slot.v1");
fn id<S: Scenario>() -> ApplicationQueryResultFieldRef<Query, Slot, S, Id<S>> {
    ApplicationQueryResultFieldRef::new("id", schema::id::<S>())
}
pub struct ValueSlot;
worth_query_portable_type!(ValueSlot => "authored-ownership.value-slot.v1");
fn parameter<S: Scenario>() -> ApplicationQueryResultFieldRef<Query, ValueSlot, S, Value<S>> {
    ApplicationQueryResultFieldRef::new("value", schema::value::<S>())
}
pub fn definition<S: Scenario>() -> ApplicationQueryDefinition<S, Query, (), Row, Node<S>> {
    let query = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(schema::node::<S>())
        .scope(schema::node::<S>())
        .result_shape({
            let shape = ApplicationQueryResultShapeBuilder::new(schema::node::<S>()).field(id());
            if matches!(S::MODE, 48 | 68 | 69 | 70) {
                shape.field(parameter()).build()
            } else {
                shape.build()
            }
        })
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public();
    let query = if matches!(S::MODE, 2 | 3) {
        query.root_path(
            ApplicationQueryRootPath::from(schema::node::<S>()).forward(Peer::reference()),
        )
    } else {
        query
    };
    query.build().unwrap()
}
impl<S: Scenario> WorthQueryApplicationProjection<S, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            id: row.field(id())?,
            value: if matches!(S::MODE, 48 | 68 | 69 | 70) {
                row.field(parameter())?
            } else {
                0
            },
        })
    }
}
pub struct Scope<S>(pub u64, pub PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationQueryScopeBinding<S> for Scope<S> {
    type Scope = Node<S>;
    type Field = Id<S>;
    type Value = u64;
    const RESOLUTION_MODE: ApplicationQueryScopeResolutionMode = if S::MODE == 1 {
        ApplicationQueryScopeResolutionMode::PrincipalIdentity
    } else {
        ApplicationQueryScopeResolutionMode::InputField
    };
}
impl<S: Scenario> ApplicationQueryScopeResolution<S, u64> for Scope<S> {
    fn into_field_parts(self, principal: &u64) -> (Id<S>, u64) {
        (
            schema::id::<S>(),
            if S::MODE == 1 { *principal } else { self.0 },
        )
    }
}
pub struct Binding<S>(PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationQueryBinding<S> for Binding<S> {
    type Input = Input;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = UnitBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = Scope<S>;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Node<S>;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "authored-ownership.source.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(16);
    fn scope_field() -> Id<S> {
        schema::id::<S>()
    }
    fn principal_binding(
    ) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node<S>, u64, U64ApplicationValueBinding>
    {
        actor()
    }
}
impl<S: Scenario> ApplicationQueryIntent<S> for Input {
    type Binding = Binding<S>;
    fn parameters(&self) -> ApplicationQueryParameterSet<Query> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> Scope<S> {
        Scope(self.id, PhantomData)
    }
}
