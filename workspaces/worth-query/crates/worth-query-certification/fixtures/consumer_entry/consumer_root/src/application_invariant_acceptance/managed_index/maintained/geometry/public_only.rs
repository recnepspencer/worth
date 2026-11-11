//! An explicit public requirement on the same genuine indirect Source, without a rule.
use super::super::{
    bounds::{Coordinates, CoordinatesBinding},
    schema::*,
};
use super::{provider, query as source, schema::Edge};
use std::sync::Arc;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::{
    application_contribution::WorthQueryApplicationContributionSetup,
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
#[cfg(test)]
mod court;

pub(super) struct PublicBoxes;
impl ApplicationManagedIndexBinding<Catalog> for PublicBoxes {
    type SourceBinding = source::SourceBinding;
    type ProbeBinding = CoordinatesBinding;
    type Entity = Edge;
    const NAME: &'static str = "public_geometry_boxes";
    const PROVIDER: &'static str = provider::PROVIDER_IDENTITY;
    const PROVIDER_VERSION: u32 = provider::PROVIDER_VERSION;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::CheckedSummarySourceTree;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}

worth_query_application_query!(pub PublicQuery for Catalog,
    identity "maintained-geometry.public-only-query.v1", parameters source::ParametersBinding,
    result source::ResultBinding, scope Directory => "Directory", name "geometry_public_only_boxes");
fn probe() -> ApplicationQueryParameterRef<PublicQuery, source::ProbeParameter, CoordinatesBinding>
{
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
fn definition() -> ApplicationQueryDefinitionBuilder<
    Catalog,
    PublicQuery,
    source::Parameters,
    source::Row,
    Directory,
> {
    ApplicationQueryDefinitionBuilder::declare(PublicQuery::reference())
        .root(Edge::reference())
        .scope(Directory::reference())
        .result_shape(source::shape())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_only())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .managed_index(PublicBoxes::reference(), probe())
        .order_by(
            source::identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindPublicBoxes {
    pub directory: String,
    pub probe: Coordinates,
}
worth_query_structured_value_binding!(pub InputBinding for FindPublicBoxes { identity: "maintained-geometry.public-only-input.v1" });
fn scope(input: FindPublicBoxes) -> String {
    input.directory
}
worth_query_query_binding!(pub Binding for FindPublicBoxes, schema Catalog,
    identity "maintained-geometry.public-only-binding.v1", input InputBinding,
    query PublicQuery, parameters source::ParametersBinding => |input:&FindPublicBoxes| ApplicationQueryParameterSet::new().bind(probe(),input.probe).unwrap(), result source::ResultBinding,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value scope, limits results 4096);

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    schema
        .application_query(definition())
        .application_query_binding::<Binding>()
        .managed_index(ApplicationManagedIndexDefinition::<Catalog, PublicBoxes>::new())
}
pub(super) fn configure(
    calls: Arc<provider::Calls>,
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.managed_index::<PublicBoxes, _>(provider::Provider(calls))
}
