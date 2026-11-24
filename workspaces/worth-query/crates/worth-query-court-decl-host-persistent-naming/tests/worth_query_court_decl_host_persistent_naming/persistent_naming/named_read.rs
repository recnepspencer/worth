//! Ordinary reads whose rows carry the persistent name of their own record:
//! `#[worth_query(name)]` projects it in the read that returns the row.
use super::*;
use worth_query_decl::facade::application_naming::WorthQueryName;
use worth_query_decl::facade::WorthQueryProjection;

/// A member with the name it carries.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = Schema, root = Member)]
pub struct NamedMember {
    #[worth_query(field = MemberKey, slot = "owned-model.named-member.key.v1")]
    pub key: String,
    #[worth_query(name)]
    pub name: WorthQueryName<Member>,
}

/// A model, whose kind has no name scheme: its optional name is absent.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = Schema, root = Model)]
pub struct MaybeNamedModel {
    #[worth_query(field = ModelKey, slot = "owned-model.maybe-named-model.key.v1")]
    pub key: String,
    #[worth_query(name)]
    pub name: Option<WorthQueryName<Model>>,
}

/// A model read through a required name slot, which an unnamed record refuses.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = Schema, root = Model)]
pub struct StrictlyNamedModel {
    #[worth_query(field = ModelKey, slot = "owned-model.strictly-named-model.key.v1")]
    pub key: String,
    #[worth_query(name)]
    pub name: WorthQueryName<Model>,
}

macro_rules! named_read { ($module:ident, $row:ident, $entity:ident, $key:ident, $semantic:literal, $identity:literal) => { pub mod $module {
use super::*;
use super::super::schema::{$entity as Entity, $key as Key};
worth_query_structured_value_binding!(pub RowBinding for $row { identity: concat!($identity, ".row") });
worth_query_application_query!(pub Query for Schema, identity concat!($identity, ".query"),
    parameters query::ParametersBinding, result RowBinding, scope Entity => $semantic, name $semantic);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let definition = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Entity::reference())
        .scope(Entity::reference())
        .result_shape($row::result_shape::<Query>())
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
worth_query_structured_value_binding!(pub ReadInput for Read { identity: concat!($identity, ".input") });
pub struct Binding;
type Scope = ApplicationQueryFieldScope<Key>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = Read;
    type InputBinding = ReadInput;
    type Query = Query;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = concat!($identity, ".binding");
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> Key {
        Key::at(stringify!($key))
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
        Scope::new(Key::at(stringify!($key)), self.0)
    }
}
} }; }
named_read!(
    member,
    NamedMember,
    Member,
    MemberKey,
    "NamedMember",
    "owned-model.named-member.v1"
);
named_read!(
    model,
    MaybeNamedModel,
    Model,
    ModelKey,
    "MaybeNamedModel",
    "owned-model.maybe-named-model.v1"
);
named_read!(
    strict_model,
    StrictlyNamedModel,
    Model,
    ModelKey,
    "StrictlyNamedModel",
    "owned-model.strictly-named-model.v1"
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    strict_model::declare(model::declare(member::declare(schema)))
}
