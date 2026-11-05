use super::*;
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Input {
    pub id: u64,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Row {
    pub id: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "authored-ownership.body.input.v1" });
worth_query_structured_value_binding!(pub RowBinding for Row { identity: "authored-ownership.body.row.v1" });
use super::super::query::UnitBinding;
worth_query_application_query!(pub Query for Schema: Scenario, identity "authored-ownership.body.read.v1", parameters UnitBinding, result RowBinding, scope Body => "Body", name "authored_body");
pub struct Slot;
worth_query_portable_type!(Slot => "authored-ownership.body.id-slot.v1");
fn id<S: Scenario>() -> ApplicationQueryResultFieldRef<
    Query,
    Slot,
    S,
    Body,
    BodyIdentity,
    BodyKey,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("id", BodyKey::reference())
}
pub fn definition<S: Scenario>() -> ApplicationQueryDefinition<S, Query, (), Row, Body> {
    let query = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Body::reference())
        .scope(Body::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::new(Body::reference())
                .field(id())
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public();
    query.build().unwrap()
}
impl<S: Scenario> WorthQueryApplicationProjection<S, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            id: row.field(id())?,
        })
    }
}
pub struct Scope<S>(pub u64, pub PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationQueryScopeBinding<S> for Scope<S> {
    type Scope = Body;
    type Aspect = BodyIdentity;
    type Field = BodyKey;
    type Value = u64;
    type Write = ReadOnly;
    type Unit = NoApplicationUnit;
    const RESOLUTION_MODE: ApplicationQueryScopeResolutionMode = if S::MODE == 1 {
        ApplicationQueryScopeResolutionMode::PrincipalIdentity
    } else {
        ApplicationQueryScopeResolutionMode::InputField
    };
}
impl<S: Scenario> ApplicationQueryScopeResolution<S, u64> for Scope<S> {
    fn into_field_parts(
        self,
        principal: &u64,
    ) -> (
        ApplicationFieldRef<
            S,
            Body,
            BodyIdentity,
            BodyKey,
            u64,
            ReadOnly,
            EqualityPredicate,
            NoApplicationUnit,
        >,
        u64,
    ) {
        (
            BodyKey::reference(),
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
    type Principal = Node;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "authored-ownership.body.source.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(16);
    fn scope_field() -> ApplicationFieldRef<
        S,
        Body,
        BodyIdentity,
        BodyKey,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }
    fn principal_binding(
    ) -> ApplicationPrincipalBindingRef<S, Actor, Mapping, Node, u64, U64ApplicationValueBinding>
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
