//! Explicit exact segment Parent on the same maintained edge Source and provider.
use super::super::{
    bounds::{Coordinates, CoordinatesBinding},
    schema::*,
};
use super::{provider::BoxIndex, query as source};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
worth_query_application_query!(pub ContactQuery for Catalog, identity "maintained-geometry.exact-contact.v1", parameters source::ParametersBinding, result source::ResultBinding, scope Directory => "Directory", name "geometry_exact_contact");
fn probe() -> ApplicationQueryParameterRef<ContactQuery, source::ProbeParameter, CoordinatesBinding>
{
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
pub fn definition() -> ApplicationQueryDefinitionBuilder<
    Catalog,
    ContactQuery,
    source::Parameters,
    source::Row,
    Directory,
> {
    source::base(
        ContactQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index_refined(BoxIndex::reference(), probe())
    .order_by(
        source::identity(),
        ApplicationQueryOrderingDirection::Ascending,
    )
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindContact {
    pub directory: String,
    pub probe: Coordinates,
}
worth_query_structured_value_binding!(pub InputBinding for FindContact {identity:"maintained-geometry.contact-input.v1"});
fn scope(input: FindContact) -> String {
    input.directory
}
worth_query_query_binding!(pub Binding for FindContact, schema Catalog, identity "maintained-geometry.contact-binding.v1", input InputBinding,
    query ContactQuery, parameters source::ParametersBinding => |input:&FindContact| ApplicationQueryParameterSet::new().bind(probe(),input.probe).unwrap(), result source::ResultBinding,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity, field DirectoryIdentity::at("DirectoryIdentity"), value scope, limits results 4096);
