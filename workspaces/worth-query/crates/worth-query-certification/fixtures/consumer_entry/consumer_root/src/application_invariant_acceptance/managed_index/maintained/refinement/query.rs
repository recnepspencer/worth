use super::*;
worth_query_application_query!(pub ExactQuery for Catalog, identity "maintained-catalog.exact-name.v1", parameters source::ParametersBinding, result source::ResultBinding, scope Directory => "Directory", name "catalog_exact_name");
pub fn probe(
) -> ApplicationQueryParameterRef<ExactQuery, source::ProbeParameter, source::ProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
pub fn definition(
) -> ApplicationQueryDefinition<Catalog, ExactQuery, source::Parameters, source::ItemRow, Directory>
{
    source::base(
        ExactQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index_refined(prefix::PrefixIndex::reference(), probe())
    .order_by(
        source::identity(),
        ApplicationQueryOrderingDirection::Ascending,
    )
    .build()
    .unwrap()
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindExact {
    pub directory: String,
    pub name: String,
}
worth_query_structured_value_binding!(pub InputBinding for FindExact { identity: "maintained-catalog.exact-input.v1" });
fn scope(input: FindExact) -> String {
    input.directory
}
worth_query_query_binding!(pub ExactBinding for FindExact, schema Catalog,
    identity "maintained-catalog.exact-binding.v1", input InputBinding,
    query ExactQuery, parameters source::ParametersBinding => |input:&FindExact| {
        ApplicationQueryParameterSet::new().bind(probe(),source::Probe {prefix:input.name.clone()}).unwrap()
    },
    result source::ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value scope, limits results 16
);
