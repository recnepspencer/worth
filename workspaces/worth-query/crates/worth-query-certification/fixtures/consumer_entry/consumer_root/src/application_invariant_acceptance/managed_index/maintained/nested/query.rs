//! Source ordering and Parent final refinement remain ordinary authored queries.
use super::{prefix::PrefixIndex, schema::*};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemRow {
    pub coordinate: (u32, u64, u32),
    pub identity: String,
    pub name: String,
    pub enabled: bool,
    pub children: Vec<super::shape::Node>,
    pub parents: Vec<super::shape::Node>,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "nested-catalog.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for ItemRow { identity: "nested-catalog.row.v1" });
worth_query_application_query!(pub SourceQuery for Catalog,
    identity "nested-catalog.source.v1", parameters ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "catalog_source");
worth_query_application_query!(pub PrefixQuery for Catalog,
    identity "nested-catalog.prefix.v1", parameters ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "catalog_prefix");

pub struct IdentitySlot;
pub struct NameSlot;
pub struct EnabledSlot;
worth_query_portable_type!(IdentitySlot => "nested-catalog.identity-slot.v1");
worth_query_portable_type!(NameSlot => "nested-catalog.name-slot.v1");
worth_query_portable_type!(EnabledSlot => "nested-catalog.enabled-slot.v1");

pub fn identity<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    IdentitySlot,
    Catalog,
    Item,
    ItemFacts,
    ItemIdentity,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", ItemIdentity::reference())
}
pub fn name<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    NameSlot,
    Catalog,
    Item,
    ItemFacts,
    ItemName,
    String,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("name", ItemName::reference())
}
pub fn enabled<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    EnabledSlot,
    Catalog,
    Item,
    ItemFacts,
    ItemEnabled,
    bool,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("enabled", ItemEnabled::reference())
}
pub(super) fn base<Q: ApplicationQueryMarkerIdentity<Catalog, ResultBinding = ResultBinding>>(
    reference: ApplicationQueryReference<Catalog, Q, Parameters, ItemRow, Directory>,
    lanes: ApplicationQueryLaneEligibility,
) -> ApplicationQueryDefinitionBuilder<Catalog, Q, Parameters, ItemRow, Directory> {
    authoring(
        reference,
        lanes,
        ApplicationQueryBasisSupport::current_and_pinned(),
    )
    .public()
}
pub(super) fn authoring<
    Q: ApplicationQueryMarkerIdentity<Catalog, ResultBinding = ResultBinding>,
>(
    reference: ApplicationQueryReference<Catalog, Q, Parameters, ItemRow, Directory>,
    lanes: ApplicationQueryLaneEligibility,
    basis: ApplicationQueryBasisSupport,
) -> ApplicationQueryAuthorizationAuthoring<Catalog, Q, Parameters, ItemRow, Directory, Item> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Item::reference())
        .scope(Directory::reference())
        .result_shape(super::shape::shape::<Q>())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(basis)
        .lanes(lanes)
}
pub fn source_definition(
) -> ApplicationQueryDefinition<Catalog, SourceQuery, Parameters, ItemRow, Directory> {
    base(
        SourceQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
    )
    .root_path(
        ApplicationQueryRootPath::from(Directory::reference()).forward(Contains::reference()),
    )
    .order_by(name(), ApplicationQueryOrderingDirection::Descending)
    .order_by(identity(), ApplicationQueryOrderingDirection::Descending)
    .build()
    .unwrap()
}

pub fn prefix_definition(
) -> ApplicationQueryDefinition<Catalog, PrefixQuery, Parameters, ItemRow, Directory> {
    base(
        PrefixQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index(PrefixIndex::reference(), probe())
    .parameter(enabled_parameter())
    .where_equal(ItemEnabled::reference(), enabled_parameter())
    .order_by(identity(), ApplicationQueryOrderingDirection::Ascending)
    .build()
    .unwrap()
}
impl<Q: ApplicationQueryMarkerIdentity<Catalog>> WorthQueryApplicationProjection<Catalog, Q>
    for ItemRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            coordinate: (
                row.entity_id().partition_value(),
                row.entity_id().local_slot_value(),
                row.entity_id().generation_value(),
            ),
            identity: row.field(identity())?,
            name: row.field(name())?,
            enabled: row.field(enabled())?,
            children: super::shape::project(row)?,
            parents: super::shape::project_parents(row)?,
        })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Probe {
    pub prefix: String,
}
worth_query_value_binding!(pub ProbeBinding for Probe {
    identity: "nested-catalog.prefix-probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(2048, 1024, 8, 8),
});
pub struct ProbeParameter;
pub struct EnabledParameter;
pub fn probe() -> ApplicationQueryParameterRef<PrefixQuery, ProbeParameter, ProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
fn enabled_parameter(
) -> ApplicationQueryParameterRef<PrefixQuery, EnabledParameter, BoolApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("enabled")
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadSource {
    pub directory: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindPrefix {
    pub directory: String,
    pub prefix: String,
}
worth_query_structured_value_binding!(pub SourceInputBinding for ReadSource { identity: "nested-catalog.source-input.v1" });
worth_query_structured_value_binding!(pub PrefixInputBinding for FindPrefix { identity: "nested-catalog.prefix-input.v1" });
fn source_scope(input: ReadSource) -> String {
    input.directory
}
fn prefix_scope(input: FindPrefix) -> String {
    input.directory
}
worth_query_query_binding!(pub SourceBinding for ReadSource, schema Catalog,
    identity "nested-catalog.source-binding.v1", input SourceInputBinding,
    query SourceQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value source_scope, limits results super::SOURCE_MAX_RESULTS
);
worth_query_query_binding!(pub PrefixBinding for FindPrefix, schema Catalog,
    identity "nested-catalog.prefix-binding.v1", input PrefixInputBinding,
    query PrefixQuery, parameters ParametersBinding => |input: &FindPrefix| {
        ApplicationQueryParameterSet::new().bind(probe(), Probe { prefix: input.prefix.clone() }).unwrap()
            .bind(enabled_parameter(), true).unwrap()
    },
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value prefix_scope, limits results 16
);
