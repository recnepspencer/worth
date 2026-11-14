//! A supported committed-state read supplies seal adoption after receipt-only replay.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct State {
    pub generation: u64,
    pub frontier: u64,
}
worth_query_structured_value_binding!(pub ResultBinding for State { identity: "owned-outputs.interleaved.seal-state" });
worth_query_application_query!(pub Query for Schema, identity "owned-outputs.interleaved.seal-read", parameters query::ParametersBinding, result ResultBinding, scope Member => "Member", name "EnvelopeSeal");
pub struct Generation;
pub struct Frontier;
worth_query_portable_type!(Generation => "owned-outputs.interleaved.generation");
worth_query_portable_type!(Frontier => "owned-outputs.interleaved.frontier");
fn generation() -> ApplicationQueryResultFieldRef<Query, Generation, Schema, EnvelopeGeneration> {
    ApplicationQueryResultFieldRef::new("generation", EnvelopeGeneration::at("Generation"))
}
fn frontier() -> ApplicationQueryResultFieldRef<Query, Frontier, Schema, EnvelopeFrontier> {
    ApplicationQueryResultFieldRef::new("frontier", EnvelopeFrontier::at("Frontier"))
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Member, State, ResultBinding>::new(
            Member::reference(),
        )
        .field(generation())
        .field(frontier())
        .build();
    schema
        .application_query(
            ApplicationQueryDefinitionBuilder::declare(Query::reference())
                .root(Member::reference())
                .scope(Member::reference())
                .result_shape(shape)
                .cardinality(ApplicationQueryCardinality::ExactlyOne)
                .disclosure(ApplicationQueryDisclosureContract::public())
                .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
                .lanes(ApplicationQueryLaneEligibility::one_shot())
                .public(),
        )
        .application_query_binding::<Binding>()
}
impl WorthQueryApplicationProjection<Schema, Query> for State {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            generation: row.field(generation())?,
            frontier: row.field(frontier())?,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read;
worth_query_structured_value_binding!(pub Input for Read { identity: "owned-outputs.interleaved.seal-read-input" });
pub struct Binding;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = Input;
    type Query = Query;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = ResultBinding;
    type ScopeBinding = ApplicationQueryFieldScope<MemberKey>;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "owned-outputs.interleaved.seal-reader";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> MemberKey {
        MemberKey::at("MemberKey")
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
    fn into_scope(self) -> ApplicationQueryFieldScope<MemberKey> {
        ApplicationQueryFieldScope::new(
            MemberKey::at("MemberKey"),
            "progress:envelope-calibration".to_owned(),
        )
    }
}
