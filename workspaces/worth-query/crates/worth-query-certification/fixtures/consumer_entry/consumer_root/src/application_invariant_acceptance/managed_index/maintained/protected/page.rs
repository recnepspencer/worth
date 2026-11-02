//! Ordinary pinned child collection pages share the existing protected Catalog root.
use super::{
    super::{prefix, query as source, schema::*},
    access,
};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Child {
    pub identity: String,
    pub name: WorthQueryApplicationDisclosed<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub directory: String,
    pub children: Vec<Child>,
}
worth_query_structured_value_binding!(pub ResultBinding for Row {identity:"maintained-catalog.protected-page-row.v1"});
worth_query_structured_value_binding!(pub ChildBinding for Child {identity:"maintained-catalog.protected-page-child.v1"});
worth_query_application_query!(pub Query for Catalog,identity "maintained-catalog.protected-page.v1",parameters source::ParametersBinding,result ResultBinding,scope Directory=>"Directory",name "protected_child_page");
worth_query_application_query!(pub BlockedQuery for Catalog,identity "maintained-catalog.blocked-page.v1",parameters source::ParametersBinding,result ResultBinding,scope Directory=>"Directory",name "blocked_child_page");
pub struct RootSlot;
pub struct ChildrenSlot;
worth_query_portable_type!(RootSlot=>"maintained-catalog.protected-page-root.v1");
worth_query_portable_type!(ChildrenSlot=>"maintained-catalog.protected-page-children.v1");
fn root<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    RootSlot,
    Catalog,
    Directory,
    DirectoryFacts,
    DirectoryIdentity,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("directory", DirectoryIdentity::reference())
}
fn children<Q>() -> ApplicationQueryResultRelationRef<
    Q,
    ChildrenSlot,
    Catalog,
    Contains,
    Directory,
    Item,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("children", Contains::reference())
}
fn builder<
    Q: ApplicationQueryMarkerIdentity<
            Catalog,
            ResultBinding = ResultBinding,
            ParameterBinding = source::ParametersBinding,
        > + 'static,
>(
    reference: ApplicationQueryReference<Catalog, Q, source::Parameters, Row, Directory>,
    name_influence: ApplicationQueryInfluenceContract,
) -> ApplicationQueryDefinitionBuilder<Catalog, Q, source::Parameters, Row, Directory> {
    let all = ApplicationQueryInfluenceContract::permit_all();
    let hidden =
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(2u64).unwrap();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "protected-child-page",
        access::ReadBoxes::reference(),
    )
    .use_field_by(
        DirectoryIdentity::reference(),
        access::encoded(),
        all.clone(),
    )
    .use_field_by(ItemIdentity::reference(), access::encoded(), all.clone())
    .use_field_by(ItemName::reference(), access::encoded(), name_influence)
    .disclose_relation_by(children::<Q>(), access::encoded(), all.clone())
    .disclose_field_by(root::<Q>(), access::encoded(), all.clone())
    .disclose_field_by(source::identity::<Q>(), access::encoded(), all.clone())
    .disclose_field_by(source::name::<Q>(), hidden, all);
    let child = ApplicationQueryResultShapeBuilder::<Catalog, Q, Item, Child, ChildBinding>::new(
        Item::reference(),
    )
    .field(source::identity())
    .field(source::name());
    let shape =
        ApplicationQueryResultShapeBuilder::<Catalog, Q, Directory, Row, ResultBinding>::new(
            Directory::reference(),
        )
        .field(root())
        .relation(children(), child)
        .build();
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Directory::reference())
        .scope(Directory::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(access::ReadBoxRows::reference())
        .order_by(source::name(), ApplicationQueryOrderingDirection::Ascending)
        .order_by(
            source::identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(children())
}
pub fn definition() -> ApplicationQueryDefinition<Catalog, Query, source::Parameters, Row, Directory>
{
    builder(
        Query::reference(),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .build()
    .unwrap()
}
pub fn blocked_definition(
) -> ApplicationQueryDefinition<Catalog, BlockedQuery, source::Parameters, Row, Directory> {
    builder(
        BlockedQuery::reference(),
        ApplicationQueryInfluenceContract::permit(
            ApplicationQueryObservableInfluence::ALL
                .into_iter()
                .filter(|s| *s != ApplicationQueryObservableInfluence::Pagination),
        ),
    )
    .build()
    .unwrap()
}
impl<Q: ApplicationQueryMarkerIdentity<Catalog>> WorthQueryApplicationProjection<Catalog, Q>
    for Row
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut values = Vec::new();
        for child in row.many(children())?.iter() {
            values.push(Child {
                identity: child.field(source::identity())?,
                name: child.disclosed_field(source::name())?,
            });
        }
        Ok(Self {
            directory: row.field(root())?,
            children: values,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadPage {
    pub directory: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadBlockedPage {
    pub directory: String,
}
worth_query_structured_value_binding!(pub InputBinding for ReadPage {identity:"maintained-catalog.protected-page-input.v1"});
worth_query_structured_value_binding!(pub BlockedInputBinding for ReadBlockedPage {identity:"maintained-catalog.blocked-page-input.v1"});
fn scope(input: ReadPage) -> String {
    input.directory
}
fn blocked_scope(input: ReadBlockedPage) -> String {
    input.directory
}
worth_query_query_binding!(pub Binding for ReadPage,schema Catalog,identity "maintained-catalog.protected-page-binding.v1",input InputBinding,
    query Query,parameters source::ParametersBinding=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
    principal CatalogPrincipal,mapping ExternalMapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Directory,DirectoryFacts,DirectoryIdentity,String,ReadOnly,NoApplicationUnit,field DirectoryIdentity::reference(),value scope,limits results 16);
worth_query_query_binding!(pub BlockedBinding for ReadBlockedPage,schema Catalog,identity "maintained-catalog.blocked-page-binding.v1",input BlockedInputBinding,
    query BlockedQuery,parameters source::ParametersBinding=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
    principal CatalogPrincipal,mapping ExternalMapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Directory,DirectoryFacts,DirectoryIdentity,String,ReadOnly,NoApplicationUnit,field DirectoryIdentity::reference(),value blocked_scope,limits results 16);

#[cfg(test)]
#[test]
fn protected_continuation_managed_shape_refuses_before_a_candidate_receiver_can_be_bypassed() {
    let probe=ApplicationQueryParameterRef::<Query,source::ProbeParameter,source::ProbeBinding>::from_query_identifier("probe");
    assert!(matches!(
        builder(
            Query::reference(),
            ApplicationQueryInfluenceContract::permit_all()
        )
        .managed_index(prefix::PrefixIndex::reference(), probe)
        .build(),
        Err(ApplicationQueryDefinitionDenial::InvalidManagedIndexShape)
    ));
    assert!(builder(
        Query::reference(),
        ApplicationQueryInfluenceContract::permit_all()
    )
    .build()
    .is_ok());
}
