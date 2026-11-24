//! Public Decl+Host courts for cold Created lifecycle, native/binary probes, and bounded watches.
mod courts;
mod provider;
mod queries;
mod schema;

use provider::{BinaryTagProvider, NativePrefixProvider};
use queries::*;
use schema::*;
use std::num::NonZeroUsize;
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::application_query::ApplicationManagedIndexDefinition;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::{application_program::*, application_schema::*, authentication::*};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::runtime::{
    ExecutionAllocationPolicy, RuntimeWorldBranchBudgetInstallation,
    RuntimeWorldBudgetInstallation, RuntimeWorldCustodyBudgetInstallation,
    RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
    RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
    RuntimeWorldRetentionBudgetInstallation, WorthQueryApplicationQueryResourceProfile,
    WorthQueryInvalidationResources, WorthQueryProductWorldClock, WorthQueryProductWorldResources,
};
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, operator_identity, request_scope,
};

pub struct CatalogFeature;
impl ApplicationFeature<CatalogSchema> for CatalogFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "managed-probe-watch.feature.v1";
}
pub struct Contracts;
impl Registration<CatalogSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<CatalogSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<CatalogSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(PrincipalBinding::reference())
            .application_query(source_query())
            .application_query_binding::<SourceBinding>()
            .application_query(find_native_query())
            .application_query_binding::<FindNativeBinding>()
            .application_query(find_binary_query())
            .application_query_binding::<FindBinaryBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<
                CatalogSchema,
                NativePrefixIndex,
            >::new())
            .managed_index(ApplicationManagedIndexDefinition::<
                CatalogSchema,
                BinaryTagIndex,
            >::new())
            .operation(rename_entry_definition())
            .application_mutation_binding::<RenameEntryBinding>())
    }
}
pub struct Contribution;
impl ApplicationSchemaContribution<CatalogSchema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("managed-probe-watch.contribution.v1");
    type Members = (Catalog, Contracts);
}
impl WorthQueryApplicationContribution<CatalogSchema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, CatalogSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup
            .managed_index_provider::<NativePrefixIndex, _>(NativePrefixProvider)
            .with_predicate_routing()
            .install()?;
        setup
            .managed_index_provider::<BinaryTagIndex, _>(BinaryTagProvider)
            .with_predicate_routing()
            .install()?;
        setup.operation_body::<RenameEntryBinding>()
    }
}
pub struct Program;
impl ApplicationProgramDefinition<CatalogSchema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("managed-probe-watch.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<CatalogSchema, CatalogFeature>()
                .native_action::<RenameEntryBinding>()
                .finish_action()
                .finish(),
        ]
    }
}
pub(super) type Host = WorthQueryProgramApplicationRuntime<CatalogSchema, Program>;

pub(super) fn limits(work_ceiling: Option<u64>) -> WorthQueryApplicationLimits {
    let clock = WorthQueryAdmissionClock::system();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, work_ceiling),
    );
    let world = WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 32,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 128,
                history_metadata_bytes: 64 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 128,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 32,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 32,
                retained_partial_metadata_bytes: 4 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 256,
                in_flight_pin_acquisition_reservations: 64,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 64,
            },
        },
        WorthQueryProductWorldClock::start(clock),
        WorthQueryInvalidationResources::new(),
        policy,
    )
    .unwrap();
    WorthQueryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(8).unwrap()),
        SignalConditionalEvaluationBudget::development(),
    )
}

pub(super) fn seed_graph(
    graph: &mut WorthQueryPrimaryGraphBootstrap<CatalogSchema>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        CatalogSchema,
    >,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(
        &installed
            .principal_binding(PrincipalBinding::reference())
            .unwrap(),
        WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
        1_u64,
        operator_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Directory::reference(),
            WorthQueryApplicationEntityKey::new("catalog").unwrap(),
        )
        .field(DirectoryKeyField::at("DirectoryKey"), "catalog".to_owned()),
    )?;
    for (key, name) in [(1, "alpha"), (2, "alpine"), (3, "beta"), (4, "gamma")] {
        let id = format!("entry-{key}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Entry::reference(),
                WorthQueryApplicationEntityKey::new(id.clone()).unwrap(),
            )
            .field(EntryKeyField::at("EntryKey"), key)
            .field(EntryNameField::at("EntryName"), name.to_owned()),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Contains::reference(),
            format!("contains-{key}"),
            WorthQueryApplicationEntityKey::new("catalog").unwrap(),
            WorthQueryApplicationEntityKey::new(id).unwrap(),
        ))?;
    }
    Ok(())
}

pub(super) fn install(work_ceiling: Option<u64>) -> Host {
    let declaration = CatalogSchema::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<CatalogSchema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse("6a6a6a6a-6a6a-6a6a-6a6a-6a6a6a6a6a6a").unwrap(),
        limits(work_ceiling),
        |_, graph, installed| seed_graph(graph, installed),
    )
    .unwrap()
}

pub(super) fn names(rows: &[EntryRow]) -> Vec<&str> {
    rows.iter().map(|row| row.name.as_str()).collect()
}

pub(super) fn rename(host: &Host, entry: u64, name: &str, key: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(RenameEntry {
            entry,
            name: name.to_owned(),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
