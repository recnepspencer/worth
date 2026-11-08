//! A protected Source feeds Public current-only Parents through both real modes.
use super::{prefix, protected::access, query as ordinary, schema::*};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::Arc;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
#[cfg(test)]
mod court;
pub(super) mod expectation;

worth_query_application_query!(pub Source for Catalog,
    identity "maintained-catalog.authorized-source.v1", parameters ordinary::ParametersBinding,
    result ordinary::ResultBinding, scope Directory => "Directory", name "authorized_source");
worth_query_application_query!(pub NativeParent for Catalog,
    identity "maintained-catalog.authorized-native-parent.v1", parameters ordinary::ParametersBinding,
    result ordinary::ResultBinding, scope Directory => "Directory", name "authorized_native_parent");
worth_query_application_query!(pub CompletedParent for Catalog,
    identity "maintained-catalog.authorized-completed-parent.v1", parameters ordinary::ParametersBinding,
    result ordinary::ResultBinding, scope Directory => "Directory", name "authorized_completed_parent");

pub struct NativeIndex;
pub struct CompletedIndex;
impl ApplicationManagedIndexBinding<Catalog> for NativeIndex {
    type SourceBinding = SourceBinding;
    type ProbeBinding = ordinary::ProbeBinding;
    type Entity = Item;
    const NAME: &'static str = "authorized_native_prefix";
    const PROVIDER: &'static str = "managed-catalog.utf8-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::CheckedSummarySourceTree;
}
impl ApplicationManagedIndexBinding<Catalog> for CompletedIndex {
    type SourceBinding = SourceBinding;
    type ProbeBinding = ordinary::ProbeBinding;
    type Entity = Item;
    const NAME: &'static str = "authorized_completed_prefix";
    const PROVIDER: &'static str = "managed-catalog.utf8-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::CheckedSummarySourceTree;
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let source = ordinary::authoring(
        Source::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
        ApplicationQueryBasisSupport::current_and_pinned(),
    )
    .requires_ability(access::ReadBoxRows::reference())
    .root_path(
        ApplicationQueryRootPath::from(Directory::reference()).forward(Contains::reference()),
    )
    .order_by(
        ordinary::name(),
        ApplicationQueryOrderingDirection::Descending,
    )
    .order_by(
        ordinary::identity(),
        ApplicationQueryOrderingDirection::Descending,
    )
    .build()
    .unwrap();
    let native = ordinary::authoring(
        NativeParent::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
        ApplicationQueryBasisSupport::current_only(),
    )
    .public()
    .managed_index(NativeIndex::reference(), probe::<NativeParent>())
    .order_by(
        ordinary::identity(),
        ApplicationQueryOrderingDirection::Ascending,
    )
    .build()
    .unwrap();
    let completed = ordinary::authoring(
        CompletedParent::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
        ApplicationQueryBasisSupport::current_only(),
    )
    .public()
    .managed_index(CompletedIndex::reference(), probe::<CompletedParent>())
    .order_by(
        ordinary::identity(),
        ApplicationQueryOrderingDirection::Ascending,
    )
    .build()
    .unwrap();
    schema
        .application_query(source)
        .application_query_binding::<SourceBinding>()
        .application_query(native)
        .application_query_binding::<NativeBinding>()
        .application_query(completed)
        .application_query_binding::<CompletedBinding>()
        .managed_index(ApplicationManagedIndexDefinition::<Catalog, NativeIndex>::new())
        .managed_index(ApplicationManagedIndexDefinition::<Catalog, CompletedIndex>::new())
        .application_mutation_binding::<expectation::NativeBinding>()
        .application_mutation_binding::<expectation::CompletedBinding>()
}
pub fn probe<Q>(
) -> ApplicationQueryParameterRef<Q, ordinary::ProbeParameter, ordinary::ProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadSource {
    pub directory: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindNative {
    pub directory: String,
    pub prefix: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindCompleted {
    pub directory: String,
    pub prefix: String,
}
worth_query_structured_value_binding!(pub SourceInputBinding for ReadSource { identity: "maintained-catalog.authorized-source-input.v1" });
worth_query_structured_value_binding!(pub NativeInputBinding for FindNative { identity: "maintained-catalog.authorized-native-input.v1" });
worth_query_structured_value_binding!(pub CompletedInputBinding for FindCompleted { identity: "maintained-catalog.authorized-completed-input.v1" });
fn source_scope(input: ReadSource) -> String {
    input.directory
}
fn native_scope(input: FindNative) -> String {
    input.directory
}
fn completed_scope(input: FindCompleted) -> String {
    input.directory
}
worth_query_query_binding!(pub SourceBinding for ReadSource, schema Catalog,
    identity "maintained-catalog.authorized-source.binding.v1", input SourceInputBinding,
    query Source, parameters ordinary::ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ordinary::ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value source_scope, limits results super::SOURCE_MAX_RESULTS);
worth_query_query_binding!(pub NativeBinding for FindNative, schema Catalog,
    identity "maintained-catalog.authorized-native.binding.v1", input NativeInputBinding,
    query NativeParent, parameters ordinary::ParametersBinding => |input: &FindNative| {
        ApplicationQueryParameterSet::new().bind(probe::<NativeParent>(), ordinary::Probe { prefix: input.prefix.clone() }).unwrap()
    },
    result ordinary::ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value native_scope, limits results 16);
worth_query_query_binding!(pub CompletedBinding for FindCompleted, schema Catalog,
    identity "maintained-catalog.authorized-completed.binding.v1", input CompletedInputBinding,
    query CompletedParent, parameters ordinary::ParametersBinding => |input: &FindCompleted| {
        ApplicationQueryParameterSet::new().bind(probe::<CompletedParent>(), ordinary::Probe { prefix: input.prefix.clone() }).unwrap()
    },
    result ordinary::ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value completed_scope, limits results 16);

#[derive(Clone, Default)]
pub(super) struct Configuration {
    pub native: Arc<prefix_observation::Calls>,
    pub completed: Arc<prefix_observation::Calls>,
}
pub(super) fn configure(
    config: Configuration,
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup
        .managed_index_provider::<NativeIndex, _>(prefix::Provider(config.native))
        .with_predicate_routing()
        .install()?;
    setup
        .managed_index_provider::<CompletedIndex, _>(prefix::Provider(config.completed))
        .with_predicate_routing()
        .install()?;
    setup.handler::<expectation::NativeBinding, _>(super::mutation::Handler)?;
    setup.handler::<expectation::CompletedBinding, _>(super::mutation::Handler)
}

impl
    crate::application_invariant_acceptance::managed_index::prefix_provider::PrefixSourceIndex<
        Catalog,
    > for NativeIndex
{
    fn with_name<R>(
        row: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
        visitor: impl FnOnce(&str) -> R,
    ) -> Result<R, WorthQueryManagedIndexTextReadDenial> {
        row.with_text(ordinary::name(), visitor)
    }
    #[cfg(test)]
    fn inspect_text(
        calls: &prefix_observation::Calls,
        row: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
    ) -> Result<(), crate::application_invariant_acceptance::managed_index::prefix_provider::Stop>
    {
        prefix::text_read::inspect(calls, row)
    }
}

impl
    crate::application_invariant_acceptance::managed_index::prefix_provider::PrefixSourceIndex<
        Catalog,
    > for CompletedIndex
{
    fn with_name<R>(
        row: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
        visitor: impl FnOnce(&str) -> R,
    ) -> Result<R, WorthQueryManagedIndexTextReadDenial> {
        row.with_text(ordinary::name(), visitor)
    }
    #[cfg(test)]
    fn inspect_text(
        calls: &prefix_observation::Calls,
        row: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
    ) -> Result<(), crate::application_invariant_acceptance::managed_index::prefix_provider::Stop>
    {
        prefix::text_read::inspect(calls, row)
    }
}
