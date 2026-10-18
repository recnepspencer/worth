//! Ordered native child pages keep admitted nested values through finalization.
use super::{output::DerivedValue, query, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

pub struct ChildIdentitySlot;
pub struct ChildValueSlot;
pub struct ChildrenSlot;
pub struct RootIdentitySlot;
worth_query_portable_type!(RootIdentitySlot => "canonical-related.live.root-identity-slot.v1");
fn root_identity() -> ApplicationQueryResultFieldRef<
    LiveQuery,
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
worth_query_portable_type!(ChildIdentitySlot => "canonical-related.live.child-identity-slot.v1");
worth_query_portable_type!(ChildValueSlot => "canonical-related.live.child-value-slot.v1");
worth_query_portable_type!(ChildrenSlot => "canonical-related.live.children-slot.v1");
worth_query_structured_value_binding!(pub LiveResultBinding for super::page_query::PageRow { identity: "canonical-related.live.result.v1" });
worth_query_structured_value_binding!(pub ChildBinding for () { identity: "canonical-related.live.child.v1" });
worth_query_application_query!(pub LiveQuery for CanonicalSchema,
    identity "canonical-related.live.query.v1", parameters query::ParametersBinding,
    result LiveResultBinding, scope Derived => "Derived", name "related_live_children");

fn child_identity() -> ApplicationQueryResultFieldRef<
    LiveQuery,
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
    LiveQuery,
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
    LiveQuery,
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

pub fn definition() -> ApplicationQueryDefinition<
    CanonicalSchema,
    LiveQuery,
    query::Parameters,
    super::page_query::PageRow,
    Derived,
> {
    let child = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        LiveQuery,
        Derived,
        (),
        ChildBinding,
    >::new(Derived::reference())
    .field(child_identity())
    .field(child_value());
    let shape = ApplicationQueryResultShapeBuilder::<
        CanonicalSchema,
        LiveQuery,
        Derived,
        super::page_query::PageRow,
        LiveResultBinding,
    >::new(Derived::reference())
    .field(root_identity())
    .relation(children(), child)
    .build();
    ApplicationQueryDefinitionBuilder::declare(LiveQuery::reference())
        .root(Derived::reference())
        .scope(Derived::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 6))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .order_by(
            child_identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .live_root_by::<super::cause::LiveCause, _, _, _, _, _, _, _, _>(
            root_identity(),
            root_identity(),
            ApplicationQueryLiveResourceContract::bounded(4, 65_536, 1_024),
        )
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<CanonicalSchema, LiveQuery> for super::page_query::PageRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, CanonicalSchema, LiveQuery>,
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
pub struct ReadLive {
    pub identity: String,
}
worth_query_structured_value_binding!(pub LiveInputBinding for ReadLive { identity: "canonical-related.live.input.v1" });
pub struct LiveBinding;
type ReadScope = ApplicationQueryFieldScope<
    CanonicalSchema,
    Derived,
    DerivedFacts,
    DerivedIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<CanonicalSchema> for LiveBinding {
    type Input = ReadLive;
    type InputBinding = LiveInputBinding;
    type Query = LiveQuery;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = LiveResultBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-related.live.binding.v1";
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
impl ApplicationQueryIntent<CanonicalSchema> for ReadLive {
    type Binding = LiveBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<LiveQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(DerivedIdentityField::reference(), self.identity)
    }
}

impl ApplicationLiveQueryIntent<CanonicalSchema> for ReadLive {
    type Target = Derived;
    type LiveCause = super::cause::LiveCause;
}
