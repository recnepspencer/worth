//! Exact index declaration shared by the real catalog compositions.
use super::{query, Catalog};
use worth_query_decl::facade::application_query::{
    ApplicationManagedIndexBinding, ApplicationManagedIndexMaintenance,
};
pub struct PrefixIndex;
// compiled-example: begin
impl ApplicationManagedIndexBinding<Catalog> for PrefixIndex {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = query::ProbeBinding;
    type Entity = super::super::schema::Item;
    const NAME: &'static str = "catalog_prefix_index";
    const PROVIDER: &'static str = "managed-catalog.utf8-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
    const MAINTENANCE: ApplicationManagedIndexMaintenance = super::super::PREFIX_MAINTENANCE;
    const REFINEMENT: Option<
        worth_query_decl::facade::application_query::ApplicationManagedIndexRefinement,
    > = super::super::PREFIX_REFINEMENT;
    const STRUCTURE: worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure =
        worth_query_decl::facade::application_query::ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP:
        worth_query_decl::facade::application_query::ApplicationManagedIndexMembership =
        super::super::PREFIX_MEMBERSHIP;
}
// compiled-example: end

/// The adapter consumes this exact entity/probe grammar; each Source's actual
/// installed projection still controls every field loan.
pub(in super::super) trait PrefixSourceIndex:
    ApplicationManagedIndexBinding<
    Catalog,
    Entity = super::super::schema::Item,
    ProbeBinding = query::ProbeBinding,
>
{
}
impl<
        I: ApplicationManagedIndexBinding<
            Catalog,
            Entity = super::super::schema::Item,
            ProbeBinding = query::ProbeBinding,
        >,
    > PrefixSourceIndex for I
{
}
