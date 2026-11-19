//! Two more Sources over the same event history, so one commit exercises
//! each reuse mode: the same ordered membership under `SameAdmittedScope`,
//! and an ordered Source whose one relation both introduces and retires
//! (Native's retirement precedence: none of its rows is ever a member).
use super::query::{self, base, ItemRow, Parameters, ParametersBinding, ResultBinding};
use super::schema::*;
#[cfg(test)]
use crate::application_invariant_acceptance::managed_index::prefix_observation::Calls;
use crate::application_invariant_acceptance::managed_index::prefix_provider::PrefixSourceIndex;
#[cfg(test)]
use crate::application_invariant_acceptance::managed_index::prefix_provider::Stop;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

/// The Source of `LiveItems` again, kept under `SameAdmittedScope`.
pub struct SameScopeIndex;
/// The Source of `LiveOverlap`, kept under `OrderedPredecessor`.
pub struct OverlapIndex;
macro_rules! prefix_binding {
    ($index:ident, $source:ty, $name:literal, $scope:ident) => {
        impl ApplicationManagedIndexBinding<Catalog> for $index {
            type SourceBinding = $source;
            type ProbeBinding = query::ProbeBinding;
            type Entity = Item;
            const NAME: &'static str = $name;
            const PROVIDER: &'static str = "managed-catalog.utf8-prefix.v1";
            const PROVIDER_VERSION: u32 = 1;
            const SOURCE_SCOPE: ApplicationManagedIndexSourceScope =
                ApplicationManagedIndexSourceScope::$scope;
            const MAINTENANCE: ApplicationManagedIndexMaintenance = super::PREFIX_MAINTENANCE;
            const REFINEMENT: Option<ApplicationManagedIndexRefinement> = super::PREFIX_REFINEMENT;
            const STRUCTURE: ApplicationManagedIndexSourceStructure =
                ApplicationManagedIndexSourceStructure::BinarySourceTree;
            const MEMBERSHIP: ApplicationManagedIndexMembership = super::PREFIX_MEMBERSHIP;
        }
        impl PrefixSourceIndex<Catalog> for $index {
            fn with_name<R>(
                row: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
                visitor: impl FnOnce(&str) -> R,
            ) -> Result<R, WorthQueryManagedIndexTextReadDenial> {
                row.with_text(query::name(), visitor)
            }
            #[cfg(test)]
            fn inspect_text(
                _: &Calls,
                _: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
            ) -> Result<(), Stop> {
                Ok(())
            }
        }
    };
}
prefix_binding!(
    SameScopeIndex,
    query::SourceBinding,
    "catalog_same_scope_index",
    SameAdmittedScope
);
prefix_binding!(
    OverlapIndex,
    OverlapBinding,
    "catalog_overlap_index",
    OrderedPredecessor
);

/// One relation in both roles of an ordered event membership.
pub struct LiveOverlap;
impl LiveOverlap {
    pub fn reference() -> ApplicationDerivedRelationRef<Catalog, Self, Directory, Item> {
        ApplicationDerivedRelationRef::from_schema_identifiers("LiveOverlap", "Directory", "Item")
    }
}
worth_query_application_query!(pub OverlapQuery for Catalog,
    identity "nested-catalog.overlap.v1", parameters ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "catalog_overlap");
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadOverlap {
    pub directory: String,
}
worth_query_structured_value_binding!(pub OverlapInputBinding for ReadOverlap { identity: "nested-catalog.overlap-input.v1" });
fn overlap_scope(input: ReadOverlap) -> String {
    input.directory
}
worth_query_query_binding!(pub OverlapBinding for ReadOverlap, schema Catalog,
    identity "nested-catalog.overlap-binding.v1", input OverlapInputBinding,
    query OverlapQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value overlap_scope, limits results super::SOURCE_MAX_RESULTS
);
fn overlap_definition(
) -> ApplicationQueryDefinitionBuilder<Catalog, OverlapQuery, Parameters, ItemRow, Directory> {
    base(
        OverlapQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
    )
    .root_path(
        ApplicationQueryRootPath::from(Directory::reference())
            .forward_derived(LiveOverlap::reference()),
    )
}
/// The overlap relation, its Source query and both extra indexes.
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    schema
        .derived_relation(
            LiveOverlap::reference(),
            ApplicationDerivedRelation::ordered_events(
                Previous::reference(),
                Contains::reference(),
                Contains::reference(),
            ),
            super::source_graph_laws(),
        )
        .application_query(overlap_definition())
        .application_query_binding::<OverlapBinding>()
        .managed_index(ApplicationManagedIndexDefinition::<Catalog, SameScopeIndex>::new())
        .managed_index(ApplicationManagedIndexDefinition::<Catalog, OverlapIndex>::new())
}
