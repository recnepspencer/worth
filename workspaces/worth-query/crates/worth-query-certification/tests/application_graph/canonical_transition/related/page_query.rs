//! Ordered native child pages keep admitted nested values through finalization.
use super::{output::DerivedValue, query, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Debug, PartialEq)]
pub struct PageRow {
    pub identity: String,
    pub children: Vec<(String, DerivedValue)>,
}
pub struct ChildIdentitySlot;
pub struct ChildValueSlot;
pub struct ChildrenSlot;
pub struct RootIdentitySlot;
worth_query_portable_type!(RootIdentitySlot => "canonical-related.root-identity-slot.v1");
fn root_identity() -> ApplicationQueryResultFieldRef<
    PageQuery,
    RootIdentitySlot,
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("root_identity", DerivedIdentityField::reference())
}
worth_query_portable_type!(ChildIdentitySlot => "canonical-related.continuation.child-identity-slot.v1");
worth_query_portable_type!(ChildValueSlot => "canonical-related.continuation.child-value-slot.v1");
worth_query_portable_type!(ChildrenSlot => "canonical-related.continuation.children-slot.v1");
worth_query_structured_value_binding!(pub PageResultBinding for PageRow { identity: "canonical-related.continuation.result.v1" });
worth_query_structured_value_binding!(pub ChildBinding for () { identity: "canonical-related.continuation.child.v1" });
worth_query_application_query!(pub PageQuery for CanonicalSchema,
    identity "canonical-related.continuation.query.v1", parameters query::ParametersBinding,
    result PageResultBinding, scope Derived => "Derived", name "related_child_page");

fn child_identity() -> ApplicationQueryResultFieldRef<
    PageQuery,
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
    ApplicationQueryResultFieldRef::new("identity", DerivedIdentityField::reference())
}
fn child_value() -> ApplicationQueryResultFieldRef<
    PageQuery,
    ChildValueSlot,
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedValueField,
    DerivedValue,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", DerivedValueField::reference())
}
fn children() -> ApplicationQueryResultRelationRef<
    PageQuery,
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

pub fn definition(
) -> ApplicationQueryDefinition<CanonicalSchema, PageQuery, query::Parameters, PageRow, Derived> {
    let child = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        PageQuery,
        Derived,
        (),
        ChildBinding,
    >::new(Derived::reference())
    .field(child_identity())
    .field(child_value());
    let shape = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        PageQuery,
        Derived,
        PageRow,
        PageResultBinding,
    >::new(Derived::reference())
    .field(root_identity())
    .relation(children(), child)
    .build();
    ApplicationQueryDefinitionBuilder::declare(PageQuery::reference())
        .root(Derived::reference())
        .scope(Derived::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 6))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(
            child_identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(children())
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<CanonicalSchema, PageQuery> for PageRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, CanonicalSchema, PageQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let identity = row.field(root_identity())?;
        let mut children_result = Vec::new();
        for child in row.many(children())?.iter() {
            let identity = child.field(child_identity())?;
            let value = child.field(child_value())?;
            // Both opaque seed and projector catch the genuine second refusal.
            // Native completion must reject their apparent success.
            let _ = child.field(child_value());
            children_result.push((identity, value));
        }
        if super::projection::returned() {
            return Err(WorthQueryApplicationProjectionDenial::reject(
                "related-page-domain",
            ));
        }
        Ok(Self {
            identity,
            children: children_result,
        })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadPage {
    pub identity: String,
}
worth_query_structured_value_binding!(pub PageInputBinding for ReadPage { identity: "canonical-related.continuation.input.v1" });
pub struct PageBinding;
type ReadScope = ApplicationQueryFieldScope<
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<CanonicalSchema> for PageBinding {
    type Input = ReadPage;
    type InputBinding = PageInputBinding;
    type Query = PageQuery;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = PageResultBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-related.continuation.binding.v1";
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
impl ApplicationQueryIntent<CanonicalSchema> for ReadPage {
    type Binding = PageBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<PageQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(DerivedIdentityField::reference(), self.identity)
    }
}
