//! Omitted Name still requires explicit permission to influence notifications.
use super::{
    super::{prefix, query as source, schema::*},
    access,
};
pub(super) mod exact;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub identity: String,
    pub name: WorthQueryApplicationDisclosed<String>,
    pub enabled: bool,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "maintained-catalog.protected-row.v1" });
worth_query_application_query!(pub Query for Catalog, identity "maintained-catalog.protected-prefix.v1", parameters source::ParametersBinding, result ResultBinding, scope Directory => "Directory", name "protected_prefix");
worth_query_application_query!(pub BlockedQuery for Catalog, identity "maintained-catalog.blocked-prefix.v1", parameters source::ParametersBinding, result ResultBinding, scope Directory => "Directory", name "blocked_prefix");
fn probe<Q>() -> ApplicationQueryParameterRef<Q, source::ProbeParameter, source::ProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
fn enabled<Q>(
) -> ApplicationQueryParameterRef<Q, source::EnabledParameter, BoolApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("enabled")
}
fn definition_for<
    Q: ApplicationQueryMarkerIdentity<
            Catalog,
            ResultBinding = ResultBinding,
            ParameterBinding = source::ParametersBinding,
        > + 'static,
>(
    reference: ApplicationQueryReference<Catalog, Q, source::Parameters, Row, Directory>,
    name_influence: ApplicationQueryInfluenceContract,
    refined: bool,
    live: bool,
) -> ApplicationQueryDefinition<Catalog, Q, source::Parameters, Row, Directory> {
    let all = ApplicationQueryInfluenceContract::permit_all();
    let hidden =
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(2u64).unwrap();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "protected-prefix",
        access::ReadBoxes::reference(),
    )
    .use_field_by(ItemIdentity::reference(), access::encoded(), all.clone())
    .use_field_by(ItemName::reference(), access::encoded(), name_influence)
    .use_field_by(ItemEnabled::reference(), access::encoded(), all.clone())
    .disclose_field_by(source::identity::<Q>(), access::encoded(), all.clone())
    .disclose_field_by(source::name::<Q>(), hidden, all.clone())
    .disclose_field_by(source::enabled::<Q>(), access::encoded(), all);
    let builder = ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Item::reference())
        .scope(Directory::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Catalog, Q, Item, Row, ResultBinding>::new(
                Item::reference(),
            )
            .field(source::identity())
            .field(source::name())
            .field(source::enabled())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(if live {
            ApplicationQueryLaneEligibility::one_shot().with_live()
        } else {
            ApplicationQueryLaneEligibility::one_shot()
        })
        .requires_ability(access::ReadBoxRows::reference());
    let builder = if refined {
        builder.managed_index_refined(prefix::PrefixIndex::reference(), probe::<Q>())
    } else {
        builder.managed_index(prefix::PrefixIndex::reference(), probe::<Q>())
    };
    builder
        .parameter(enabled::<Q>())
        .where_equal(ItemEnabled::reference(), enabled::<Q>())
        .order_by(
            source::identity::<Q>(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .unwrap()
}
pub fn definition() -> ApplicationQueryDefinition<Catalog, Query, source::Parameters, Row, Directory>
{
    definition_for(
        Query::reference(),
        ApplicationQueryInfluenceContract::permit_all(),
        false,
        true,
    )
}
pub fn blocked_definition(
) -> ApplicationQueryDefinition<Catalog, BlockedQuery, source::Parameters, Row, Directory> {
    definition_for(
        BlockedQuery::reference(),
        ApplicationQueryInfluenceContract::permit(
            ApplicationQueryObservableInfluence::ALL
                .into_iter()
                .filter(|surface| *surface != ApplicationQueryObservableInfluence::LiveMembership),
        ),
        false,
        true,
    )
}
impl<Q: ApplicationQueryMarkerIdentity<Catalog>> WorthQueryApplicationProjection<Catalog, Q>
    for Row
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(source::identity())?,
            name: row.disclosed_field(source::name())?,
            enabled: row.field(source::enabled())?,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Find {
    pub directory: String,
    pub prefix: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindAlias {
    pub directory: String,
    pub prefix: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindBlocked {
    pub directory: String,
    pub prefix: String,
}
worth_query_structured_value_binding!(pub InputBinding for Find { identity: "maintained-catalog.protected-prefix-input.v1" });
worth_query_structured_value_binding!(pub AliasInputBinding for FindAlias { identity: "maintained-catalog.protected-prefix-alias-input.v1" });
worth_query_structured_value_binding!(pub BlockedInputBinding for FindBlocked { identity: "maintained-catalog.blocked-prefix-input.v1" });
fn scope(input: Find) -> String {
    input.directory
}
fn alias_scope(input: FindAlias) -> String {
    input.directory
}
fn blocked_scope(input: FindBlocked) -> String {
    input.directory
}
worth_query_query_binding!(pub Binding for Find, schema Catalog, identity "maintained-catalog.protected-prefix-binding.v1", input InputBinding,
    query Query, parameters source::ParametersBinding => |input: &Find| { ApplicationQueryParameterSet::new()
        .bind(probe::<Query>(), source::Probe { prefix: input.prefix.clone() }).unwrap().bind(enabled::<Query>(), true).unwrap() },
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value scope, limits results 16);
worth_query_query_binding!(pub AliasBinding for FindAlias, schema Catalog, identity "maintained-catalog.protected-alias-prefix-binding.v1", input AliasInputBinding,
    query Query, parameters source::ParametersBinding => |input: &FindAlias| { ApplicationQueryParameterSet::new()
        .bind(probe::<Query>(), source::Probe { prefix: input.prefix.clone() }).unwrap().bind(enabled::<Query>(), true).unwrap() },
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value alias_scope, limits results 16);
worth_query_query_binding!(pub BlockedBinding for FindBlocked, schema Catalog, identity "maintained-catalog.blocked-prefix-binding.v1", input BlockedInputBinding,
    query BlockedQuery, parameters source::ParametersBinding => |input: &FindBlocked| { ApplicationQueryParameterSet::new()
        .bind(probe::<BlockedQuery>(), source::Probe { prefix: input.prefix.clone() }).unwrap().bind(enabled::<BlockedQuery>(), true).unwrap() },
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value blocked_scope, limits results 16);
