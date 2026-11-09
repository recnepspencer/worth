//! Non-Live exact membership still requires every implicit input influence.
use super::*;
worth_query_application_query!(pub Query for Catalog, identity "maintained-catalog.protected-exact.v1", parameters source::ParametersBinding, result ResultBinding, scope Directory => "Directory", name "protected_exact");
worth_query_application_query!(pub BlockedQuery for Catalog, identity "maintained-catalog.blocked-exact.v1", parameters source::ParametersBinding, result ResultBinding, scope Directory => "Directory", name "blocked_exact");
pub fn definition(
) -> ApplicationQueryDefinitionBuilder<Catalog, Query, source::Parameters, Row, Directory> {
    definition_for(
        Query::reference(),
        ApplicationQueryInfluenceContract::permit_all(),
        true,
        false,
    )
}
pub fn blocked_definition(
) -> ApplicationQueryDefinitionBuilder<Catalog, BlockedQuery, source::Parameters, Row, Directory> {
    definition_for(
        BlockedQuery::reference(),
        ApplicationQueryInfluenceContract::permit(
            ApplicationQueryObservableInfluence::ALL
                .into_iter()
                .filter(|s| *s != ApplicationQueryObservableInfluence::RowPresence),
        ),
        true,
        false,
    )
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Find {
    pub directory: String,
    pub prefix: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindBlocked {
    pub directory: String,
    pub prefix: String,
}
worth_query_structured_value_binding!(pub InputBinding for Find { identity: "maintained-catalog.protected-exact-input.v1" });
worth_query_structured_value_binding!(pub BlockedInputBinding for FindBlocked { identity: "maintained-catalog.blocked-exact-input.v1" });
fn scope(input: Find) -> String {
    input.directory
}
fn blocked_scope(input: FindBlocked) -> String {
    input.directory
}
worth_query_query_binding!(pub Binding for Find, schema Catalog, identity "maintained-catalog.protected-exact-binding.v1", input InputBinding,
    query Query, parameters source::ParametersBinding => |input: &Find| { ApplicationQueryParameterSet::new()
        .bind(probe::<Query>(), source::Probe { prefix: input.prefix.clone() }).unwrap().bind(enabled::<Query>(), true).unwrap() },
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value scope, limits results 16);
worth_query_query_binding!(pub BlockedBinding for FindBlocked, schema Catalog, identity "maintained-catalog.blocked-exact-binding.v1", input BlockedInputBinding,
    query BlockedQuery, parameters source::ParametersBinding => |input: &FindBlocked| { ApplicationQueryParameterSet::new()
        .bind(probe::<BlockedQuery>(), source::Probe { prefix: input.prefix.clone() }).unwrap().bind(enabled::<BlockedQuery>(), true).unwrap() },
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value blocked_scope, limits results 16);
