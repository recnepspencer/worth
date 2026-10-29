//! Actual complete Source, Public box Parent and capability-governed box Parent.
use super::super::{
    query::{enabled, identity},
    schema::*,
};
use super::{
    access,
    coordinates::{Binding as CoordinatesBinding, Coordinates},
    field::*,
    provider::BoxIndex,
};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub coordinate: (u32, u64, u32),
    pub identity: String,
    pub bounds: Coordinates,
    pub enabled: bool,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "managed-catalog.box.parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "managed-catalog.box.row.v1" });
worth_query_application_query!(pub SourceQuery for Catalog, identity "managed-catalog.box.source.v1", parameters ParametersBinding, result ResultBinding, scope Directory => "Directory", name "catalog_box_source");
worth_query_application_query!(pub BoxQuery for Catalog, identity "managed-catalog.box.public.v1", parameters ParametersBinding, result ResultBinding, scope Directory => "Directory", name "catalog_box_public");
worth_query_application_query!(pub GovernedQuery for Catalog, identity "managed-catalog.box.governed.v1", parameters ParametersBinding, result ResultBinding, scope Directory => "Directory", name "catalog_box_governed");
pub struct BoundsSlot;
pub struct ProbeParameter;
pub struct EnabledParameter;
worth_query_portable_type!(BoundsSlot => "managed-catalog.box.bounds-slot.v1");
pub fn bounds<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    BoundsSlot,
    Catalog,
    Item,
    ItemFacts,
    ItemBounds,
    Coordinates,
    ReadOnly,
    EqualityPredicate,
    DeclaredApplicationUnit<FixtureLength, ()>,
> {
    ApplicationQueryResultFieldRef::new("bounds", ItemBounds::reference())
}
fn shape<Q: ApplicationQueryMarkerIdentity<Catalog>>(
) -> TypedApplicationQueryResultShape<Catalog, Q, Item, Row, ResultBinding> {
    ApplicationQueryResultShapeBuilder::new(Item::reference())
        .field(identity())
        .field(bounds())
        .field(enabled())
        .build()
}
fn base<Q: ApplicationQueryMarkerIdentity<Catalog, ResultBinding = ResultBinding>>(
    reference: ApplicationQueryReference<Catalog, Q, Parameters, Row, Directory>,
    disclosure: ApplicationQueryDisclosureContract,
) -> ApplicationQueryAuthorizationAuthoring<Catalog, Q, Parameters, Row, Directory, Item> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Item::reference())
        .scope(Directory::reference())
        .result_shape(shape())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
}
pub fn source_definition(
) -> ApplicationQueryDefinition<Catalog, SourceQuery, Parameters, Row, Directory> {
    base(
        SourceQuery::reference(),
        ApplicationQueryDisclosureContract::public(),
    )
    .public()
    .root_path(
        ApplicationQueryRootPath::from(Directory::reference()).forward(Contains::reference()),
    )
    .order_by(identity(), ApplicationQueryOrderingDirection::Descending)
    .build()
    .unwrap()
}
fn probe<Q>() -> ApplicationQueryParameterRef<Q, ProbeParameter, CoordinatesBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
fn enabled_parameter<Q>(
) -> ApplicationQueryParameterRef<Q, EnabledParameter, BoolApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("enabled")
}
pub fn public_definition(
) -> ApplicationQueryDefinition<Catalog, BoxQuery, Parameters, Row, Directory> {
    base(
        BoxQuery::reference(),
        ApplicationQueryDisclosureContract::public(),
    )
    .public()
    .managed_index(BoxIndex::reference(), probe())
    .parameter(enabled_parameter())
    .where_equal(ItemEnabled::reference(), enabled_parameter())
    .order_by(identity(), ApplicationQueryOrderingDirection::Ascending)
    .build()
    .unwrap()
}
pub fn governed_definition(
) -> ApplicationQueryDefinition<Catalog, GovernedQuery, Parameters, Row, Directory> {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "catalog-box-query",
        access::ReadBoxes::reference(),
    )
    .use_field_by(
        ItemIdentity::reference(),
        access::encoded(),
        influence.clone(),
    )
    .use_field_by(
        ItemEnabled::reference(),
        access::encoded(),
        influence.clone(),
    )
    .disclose_field_by(
        identity::<GovernedQuery>(),
        access::encoded(),
        influence.clone(),
    )
    .disclose_field_by(
        bounds::<GovernedQuery>(),
        access::encoded(),
        influence.clone(),
    )
    .disclose_field_by(enabled::<GovernedQuery>(), access::encoded(), influence);
    base(GovernedQuery::reference(), disclosure)
        .requires_ability(access::ReadBoxRows::reference())
        .managed_index(BoxIndex::reference(), probe())
        .parameter(enabled_parameter())
        .where_equal(ItemEnabled::reference(), enabled_parameter())
        .order_by(identity(), ApplicationQueryOrderingDirection::Ascending)
        .build()
        .unwrap()
}
impl<Q: ApplicationQueryMarkerIdentity<Catalog>> WorthQueryApplicationProjection<Catalog, Q>
    for Row
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let id = row.entity_id();
        Ok(Self {
            coordinate: (
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value(),
            ),
            identity: row.field(identity())?,
            bounds: row.field(bounds())?,
            enabled: row.field(enabled())?,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadSource {
    pub directory: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindBoxes {
    pub directory: String,
    pub probe: Coordinates,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindGovernedBoxes {
    pub directory: String,
    pub probe: Coordinates,
}
worth_query_structured_value_binding!(pub SourceInputBinding for ReadSource { identity: "managed-catalog.box.source-input.v1" });
worth_query_structured_value_binding!(pub PublicInputBinding for FindBoxes { identity: "managed-catalog.box.public-input.v1" });
worth_query_structured_value_binding!(pub GovernedInputBinding for FindGovernedBoxes { identity: "managed-catalog.box.governed-input.v1" });
fn source_scope(input: ReadSource) -> String {
    input.directory
}
fn public_scope(input: FindBoxes) -> String {
    input.directory
}
fn governed_scope(input: FindGovernedBoxes) -> String {
    input.directory
}
worth_query_query_binding!(pub SourceBinding for ReadSource, schema Catalog, identity "managed-catalog.box.source-binding.v1", input SourceInputBinding,
    query SourceQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ResultBinding,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit, field DirectoryIdentity::reference(), value source_scope, limits results 16);
worth_query_query_binding!(pub PublicBinding for FindBoxes, schema Catalog, identity "managed-catalog.box.public-binding.v1", input PublicInputBinding,
    query BoxQuery, parameters ParametersBinding => |input:&FindBoxes| ApplicationQueryParameterSet::new().bind(probe(),input.probe).unwrap().bind(enabled_parameter(),true).unwrap(), result ResultBinding,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit, field DirectoryIdentity::reference(), value public_scope, limits results 16);
worth_query_query_binding!(pub GovernedBinding for FindGovernedBoxes, schema Catalog, identity "managed-catalog.box.governed-binding.v1", input GovernedInputBinding,
    query GovernedQuery, parameters ParametersBinding => |input:&FindGovernedBoxes| ApplicationQueryParameterSet::new().bind(probe(),input.probe).unwrap().bind(enabled_parameter(),true).unwrap(), result ResultBinding,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit, field DirectoryIdentity::reference(), value governed_scope, limits results 16);
