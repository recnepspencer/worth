//! Ordinary reverse and convergent multi-step reads use the same named recipe.
use super::{query, schema::*};
use query::{ItemRow, Parameters, ParametersBinding, ResultBinding};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
worth_query_application_query!(pub RoundTripQuery for Catalog,identity "event-catalog.roundtrip.v1",parameters ParametersBinding,
    result ResultBinding,scope Directory=>"Directory",name "event_roundtrip");
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct RoundTrip {
    pub directory: String,
}
worth_query_structured_value_binding!(pub RoundInputBinding for RoundTrip {identity:"event-catalog.round-input.v1"});
fn round_scope(input: RoundTrip) -> String {
    input.directory
}
worth_query_query_binding!(pub RoundBinding for RoundTrip,schema Catalog,identity "event-catalog.round-binding.v1",input RoundInputBinding,
    query RoundTripQuery,parameters ParametersBinding=>|_|ApplicationQueryParameterSet::new(),result ResultBinding,
    principal CatalogPrincipal,mapping ExternalMapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity, field DirectoryIdentity::at("DirectoryIdentity"),value round_scope,limits results 8192);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRow {
    pub identity: String,
    pub coordinate: (u32, u64, u32),
}
worth_query_structured_value_binding!(pub EventResultBinding for EventRow {identity:"event-catalog.event-row.v1"});
worth_query_application_query!(pub InverseQuery for Catalog,identity "event-catalog.inverse.v1",parameters ParametersBinding,
    result EventResultBinding,scope Item=>"Item",name "event_inverse");
pub struct EventIdentitySlot;
worth_query_portable_type!(EventIdentitySlot=>"event-catalog.event-identity-slot.v1");
fn event_identity(
) -> ApplicationQueryResultFieldRef<InverseQuery, EventIdentitySlot, Catalog, DirectoryIdentity> {
    ApplicationQueryResultFieldRef::new("identity", DirectoryIdentity::at("DirectoryIdentity"))
}
impl WorthQueryApplicationProjection<Catalog, InverseQuery> for EventRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, InverseQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(event_identity())?,
            coordinate: (
                row.entity_id().partition_value(),
                row.entity_id().local_slot_value(),
                row.entity_id().generation_value(),
            ),
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Inverse {
    pub item: String,
}
worth_query_structured_value_binding!(pub InverseInputBinding for Inverse {identity:"event-catalog.inverse-input.v1"});
fn inverse_scope(input: Inverse) -> String {
    input.item
}
worth_query_query_binding!(pub InverseBinding for Inverse,schema Catalog,identity "event-catalog.inverse-binding.v1",input InverseInputBinding,
    query InverseQuery,parameters ParametersBinding=>|_|ApplicationQueryParameterSet::new(),result EventResultBinding,
    principal CatalogPrincipal,mapping ExternalMapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope ItemIdentity, field ItemIdentity::at("ItemIdentity"),value inverse_scope,limits results 8192);
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let round: ApplicationQueryDefinition<Catalog, RoundTripQuery, Parameters, ItemRow, Directory> =
        query::base(
            RoundTripQuery::reference(),
            ApplicationQueryLaneEligibility::one_shot(),
        )
        .root_path(
            ApplicationQueryRootPath::from(Directory::reference())
                .forward_derived(LiveItems::reference())
                .reverse_derived(LiveItems::reference())
                .forward_derived(LiveItems::reference()),
        )
        .order_by(
            query::identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .unwrap();
    let shape = ApplicationQueryResultShapeBuilder::<
        Catalog,
        InverseQuery,
        Directory,
        EventRow,
        EventResultBinding,
    >::new(Directory::reference())
    .field(event_identity())
    .build();
    let inverse = ApplicationQueryDefinitionBuilder::declare(InverseQuery::reference())
        .root(Directory::reference())
        .scope(Item::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Item::reference())
                .reverse_derived(LiveItems::reference()),
        )
        .order_by(
            event_identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .unwrap();
    schema
        .application_query(round)
        .application_query_binding::<RoundBinding>()
        .application_query(inverse)
        .application_query_binding::<InverseBinding>()
}
