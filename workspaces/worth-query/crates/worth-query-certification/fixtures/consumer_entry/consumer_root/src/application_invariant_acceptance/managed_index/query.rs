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
}
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "managed-catalog.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for ItemRow { identity: "managed-catalog.row.v1" });
worth_query_application_query!(pub SourceQuery for Catalog,
    identity "managed-catalog.source.v1", parameters ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "catalog_source");
worth_query_application_query!(pub PrefixQuery for Catalog,
    identity "managed-catalog.prefix.v1", parameters ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "catalog_prefix");

pub struct IdentitySlot;
pub struct NameSlot;
pub struct EnabledSlot;
worth_query_portable_type!(IdentitySlot => "managed-catalog.identity-slot.v1");
worth_query_portable_type!(NameSlot => "managed-catalog.name-slot.v1");
worth_query_portable_type!(EnabledSlot => "managed-catalog.enabled-slot.v1");

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
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("enabled", ItemEnabled::reference())
}
fn base<Q: ApplicationQueryMarkerIdentity<Catalog, ResultBinding = ResultBinding>>(
    reference: ApplicationQueryReference<Catalog, Q, Parameters, ItemRow, Directory>,
) -> ApplicationQueryDefinitionBuilder<Catalog, Q, Parameters, ItemRow, Directory> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Item::reference())
        .scope(Directory::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Catalog, Q, Item, ItemRow, ResultBinding>::new(
                Item::reference(),
            )
            .field(identity())
            .field(name())
            .field(enabled())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
pub fn source_definition(
) -> ApplicationQueryDefinition<Catalog, SourceQuery, Parameters, ItemRow, Directory> {
    base(SourceQuery::reference())
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
    base(PrefixQuery::reference())
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
        })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Probe {
    pub prefix: String,
}
worth_query_value_binding!(pub ProbeBinding for Probe {
    identity: "managed-catalog.prefix-probe.v1", scalar: String,
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
worth_query_structured_value_binding!(pub SourceInputBinding for ReadSource { identity: "managed-catalog.source-input.v1" });
worth_query_structured_value_binding!(pub PrefixInputBinding for FindPrefix { identity: "managed-catalog.prefix-input.v1" });
fn source_scope(input: ReadSource) -> String {
    input.directory
}
fn prefix_scope(input: FindPrefix) -> String {
    input.directory
}
worth_query_query_binding!(pub SourceBinding for ReadSource, schema Catalog,
    identity "managed-catalog.source-binding.v1", input SourceInputBinding,
    query SourceQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value source_scope, limits results 16
);
worth_query_query_binding!(pub PrefixBinding for FindPrefix, schema Catalog,
    identity "managed-catalog.prefix-binding.v1", input PrefixInputBinding,
    query PrefixQuery, parameters ParametersBinding => |input: &FindPrefix| {
        ApplicationQueryParameterSet::new().bind(probe(), Probe { prefix: input.prefix.clone() }).unwrap()
            .bind(enabled_parameter(), true).unwrap()
    },
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value prefix_scope, limits results 16
);
