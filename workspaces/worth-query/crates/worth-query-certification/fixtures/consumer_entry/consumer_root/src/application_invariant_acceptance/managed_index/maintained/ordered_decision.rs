//! An ordered-events Parent beside the catalog's direct-membership Parent.
use super::{query, schema::*};
use crate::application_invariant_acceptance::managed_index::prefix_provider::PrefixSourceIndex;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

pub struct Previous;
impl fragment::Relation for Previous {
    const NAME: &'static str = "Previous";
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, Some(1), None, None, None, None);
    type To = Directory;
}
impl Previous {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Directory, Directory> {
        fragment::relation(CatalogRecords, Directory, Self)
    }
}

pub struct Entries;
impl fragment::Relation for Entries {
    const NAME: &'static str = "OrderedEntries";
    type To = Item;
}
impl Entries {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Directory, Item> {
        fragment::relation(CatalogRecords, Directory, Self)
    }
}

pub struct Retires;
impl fragment::Relation for Retires {
    const NAME: &'static str = "Retires";
    type To = Item;
}
impl Retires {
    fn reference() -> ApplicationRelationRef<Catalog, Self, Directory, Item> {
        fragment::relation(CatalogRecords, Directory, Self)
    }
}

struct LiveItems;
impl LiveItems {
    fn reference() -> ApplicationDerivedRelationRef<Catalog, Self, Directory, Item> {
        ApplicationDerivedRelationRef::from_schema_identifiers("LiveItems", "Directory", "Item")
    }
}

use query::{ItemRow, Parameters, ParametersBinding, ResultBinding};

std::thread_local! {
    pub static SEED_ENTRY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
worth_query_application_query!(pub Source for Catalog,
    identity "managed-catalog.ordered-decision-source.v1", parameters ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "catalog_ordered_decision_source");
worth_query_application_query!(pub Parent for Catalog,
    identity "managed-catalog.ordered-decision-parent.v1", parameters ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "catalog_ordered_decision_parent");

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadSource {
    pub directory: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Find {
    pub directory: String,
    pub prefix: String,
}
worth_query_structured_value_binding!(pub SourceInput for ReadSource { identity: "managed-catalog.ordered-decision-source-input.v1" });
worth_query_structured_value_binding!(pub ParentInput for Find { identity: "managed-catalog.ordered-decision-parent-input.v1" });
fn scope(input: ReadSource) -> String {
    input.directory
}
fn probe_scope(input: Find) -> String {
    input.directory
}
worth_query_query_binding!(pub SourceBinding for ReadSource, schema Catalog,
    identity "managed-catalog.ordered-decision-source-binding.v1", input SourceInput,
    query Source, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value scope, limits results 16
);
worth_query_query_binding!(pub ParentBinding for Find, schema Catalog,
    identity "managed-catalog.ordered-decision-parent-binding.v1", input ParentInput,
    query Parent, parameters ParametersBinding => |input: &Find| {
        ApplicationQueryParameterSet::new().bind(probe(), query::Probe { prefix: input.prefix.clone() }).unwrap()
    },
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value probe_scope, limits results 16
);

fn probe() -> ApplicationQueryParameterRef<Parent, query::ProbeParameter, query::ProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}

pub struct Index;
impl ApplicationManagedIndexBinding<Catalog> for Index {
    type SourceBinding = SourceBinding;
    type ProbeBinding = query::ProbeBinding;
    type Entity = Item;
    const NAME: &'static str = "catalog_ordered_decision_index";
    const PROVIDER: &'static str = "managed-catalog.utf8-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
    const SOURCE_SCOPE: ApplicationManagedIndexSourceScope =
        ApplicationManagedIndexSourceScope::OrderedPredecessor;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::CheckedSummarySourceTree;
}
impl PrefixSourceIndex<Catalog> for Index {
    fn with_name<R>(
        row: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
        visitor: impl FnOnce(&str) -> R,
    ) -> Result<R, WorthQueryManagedIndexTextReadDenial> {
        row.with_text(query::name(), visitor)
    }

    #[cfg(test)]
    fn inspect_text(
        _: &crate::application_invariant_acceptance::managed_index::prefix_observation::Calls,
        _: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
    ) -> Result<(), crate::application_invariant_acceptance::managed_index::prefix_provider::Stop>
    {
        Ok(())
    }
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let source: ApplicationQueryDefinitionBuilder<Catalog, Source, Parameters, ItemRow, Directory> =
        query::base(
            Source::reference(),
            ApplicationQueryLaneEligibility::one_shot(),
        )
        .root_path(
            ApplicationQueryRootPath::from(Directory::reference())
                .forward_derived(LiveItems::reference()),
        )
        .order_by(
            query::identity(),
            ApplicationQueryOrderingDirection::Ascending,
        );
    let parent = query::base(
        Parent::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
    )
    .managed_index(Index::reference(), probe())
    .order_by(
        query::identity(),
        ApplicationQueryOrderingDirection::Ascending,
    );
    schema
        .derived_relation(
            LiveItems::reference(),
            ApplicationDerivedRelation::ordered_events(
                Previous::reference(),
                Entries::reference(),
                Retires::reference(),
            ),
            ApplicationRelationGraphLaws::unbounded(),
        )
        .application_query(source)
        .application_query_binding::<SourceBinding>()
        .application_query(parent)
        .application_query_binding::<ParentBinding>()
        .managed_index(ApplicationManagedIndexDefinition::<Catalog, Index>::new())
}
