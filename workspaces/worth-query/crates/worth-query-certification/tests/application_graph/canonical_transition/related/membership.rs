//! Real root/child membership, with no canonical value decoded during its census.
use super::{derived_query as query, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
use worth_relational::facade::identity::EntityId;

pub struct Members {
    pub rows: Vec<(EntityId, String)>,
}
pub struct IdentitySlot;
pub struct ChildIdentitySlot;
pub struct ChildrenSlot;
worth_query_portable_type!(IdentitySlot => "canonical-related.paired.identity-slot.v1");
worth_query_portable_type!(ChildIdentitySlot => "canonical-related.paired.child-identity-slot.v1");
worth_query_portable_type!(ChildrenSlot => "canonical-related.paired.children-slot.v1");
worth_query_structured_value_binding!(pub MembersBinding for Members { identity: "canonical-related.paired.members.v1" });
worth_query_structured_value_binding!(pub ChildBinding for () { identity: "canonical-related.paired.member-child.v1" });
worth_query_application_query!(pub MembershipQuery for CanonicalSchema,
    identity "canonical-related.paired.membership.v1", parameters query::ParametersBinding,
    result MembersBinding, scope Derived => "Derived", name "paired_canonical_membership");

fn identity() -> ApplicationQueryResultFieldRef<
    MembershipQuery,
    IdentitySlot,
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", DerivedIdentityField::reference())
}

fn children() -> ApplicationQueryResultRelationRef<
    MembershipQuery,
    ChildrenSlot,
    CanonicalSchema,
    DerivedChildren,
    Derived,
    Derived,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("children", DerivedChildren::reference())
}

fn child_identity() -> ApplicationQueryResultFieldRef<
    MembershipQuery,
    ChildIdentitySlot,
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("child_identity", DerivedIdentityField::reference())
}

pub fn definition(
) -> ApplicationQueryDefinition<CanonicalSchema, MembershipQuery, query::Parameters, Members, Derived>
{
    let child = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        MembershipQuery,
        Derived,
        (),
        ChildBinding,
    >::new(Derived::reference())
    .field(child_identity());
    let shape = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        MembershipQuery,
        Derived,
        Members,
        MembersBinding,
    >::new(Derived::reference())
    .field(identity())
    .relation(children(), child)
    .build();
    ApplicationQueryDefinitionBuilder::declare(MembershipQuery::reference())
        .root(Derived::reference())
        .scope(Derived::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<CanonicalSchema, MembershipQuery> for Members {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, CanonicalSchema, MembershipQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut rows = vec![(row.entity_id(), row.field(identity())?)];
        for child in row.many(children())?.iter() {
            rows.push((child.entity_id(), child.field(child_identity())?));
        }
        Ok(Self { rows })
    }
}

pub struct MembershipBinding;
type ReadScope = ApplicationQueryFieldScope<
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<CanonicalSchema> for MembershipBinding {
    type Input = query::ReadValue;
    type InputBinding = query::ReadInputBinding;
    type Query = MembershipQuery;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = MembersBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-related.paired.membership-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        CanonicalSchema,
        Derived,
        DerivedFacts,
        DerivedIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DerivedIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        CanonicalSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
