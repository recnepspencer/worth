//! A partitioned managed computation declared, installed and run through the
//! public facade: its computation partition key and determinism contract are
//! program meaning, and its execution posture decides which key it may declare
//! and which owner binding serves it.

use std::marker::PhantomData;

use serde::Serialize;
use worth_query_decl::facade::application_program::{
    ApplicationComputationExecution, ApplicationComputationInput, ApplicationComputationPartition,
    ApplicationComputationResourceCeiling, ApplicationComputationReuse,
    ApplicationComputationStopped, ApplicationManagedComputation, ApplicationProgramRevision,
    ApplicationProgramValidationDenialKind, ApplicationSingleComputationPartition,
    DeterminismContract, EquivalenceContractId,
};
use worth_query_decl::facade::application_schema::{
    ApplicationSchemaContribution, ApplicationSchemaContributionIdentity,
    ApplicationSchemaDeclarationBuilder,
};
use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationContribution, WorthQueryApplicationContributionContracts,
    WorthQueryApplicationContributionSetup,
};

use super::*;

mod compiled_examples;
mod demand;
mod entry_correction;
mod entry_edit;
mod execution;
mod facts;
#[cfg(feature = "test-query-execution-observer")]
mod oracle;
#[cfg(feature = "test-query-execution-observer")]
mod output_producer;
mod owner;
mod probe;
mod refusal;
mod region_output;
#[cfg(feature = "test-query-execution-observer")]
mod worker_axis;

/// A set of entries, each tagged with the region it lies in. An owner reads
/// the entries from the set's facts.
struct RegionEntries;
impl ApplicationComputationInput for RegionEntries {
    type Value = facts::Set;
    const IDENTITY: &'static str = "checkpoint-region-entries";
}

// A key is the same key when its canonical encoding is: it needs no ordering
// or equality of its own.
macro_rules! region_key {
    ($key:ident, $identity:literal) => {
        #[derive(Clone, Copy, Serialize)]
        struct $key(u32);
        impl ApplicationComputationPartition for $key {
            const IDENTITY: &'static str = $identity;
        }
    };
}
region_key!(RegionKey, "checkpoint-region-key");
region_key!(RenamedRegionKey, "checkpoint-renamed-region-key");

struct NoWarmStart;
impl ApplicationComputationReuse for NoWarmStart {
    const IDENTITY: &'static str = "checkpoint-region-no-warm-start";
}
struct RegionStopped;
impl ApplicationComputationStopped for RegionStopped {
    const IDENTITY: &'static str = "checkpoint-region-totals-stopped";
}

// Every variant keeps one computation identity, so two programs differ only
// in the declared member under test.
macro_rules! region_totals {
    ($computation:ident, $partition:ty, $execution:ident, bytes $bytes:expr $(, $determinism:expr)?) => {
        struct $computation;
        impl ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>
            for $computation
        {
            type Input = RegionEntries;
            type Output = demand_policy::FinalArtifact;
            type Partition = $partition;
            type Reuse = NoWarmStart;
            type Stopped = RegionStopped;
            const IDENTITY: &'static str = "checkpoint-region-totals";
            const EXECUTION: ApplicationComputationExecution =
                ApplicationComputationExecution::$execution;
            $(const DETERMINISM: DeterminismContract = $determinism;)?
            const RESOURCES: ApplicationComputationResourceCeiling =
                ApplicationComputationResourceCeiling::new(4_096, $bytes);
        }
    };
    ($computation:ident, $partition:ty, $execution:ident $(, $determinism:expr)?) => {
        region_totals!($computation, $partition, $execution, bytes 8_192 $(, $determinism)?);
    };
}
region_totals!(RegionTotals, RegionKey, DeterministicPartitioned);
region_totals!(RenamedKeyTotals, RenamedRegionKey, DeterministicPartitioned);
region_totals!(
    EquivalentTotals,
    RegionKey,
    DeterministicPartitioned,
    DeterminismContract::ContractEquivalent(EquivalenceContractId::new(1))
);
region_totals!(
    SingleKeyPartitionedTotals,
    ApplicationSingleComputationPartition,
    DeterministicPartitioned
);
region_totals!(KeyedSerialTotals, RegionKey, Deterministic);
region_totals!(
    SerialTotals,
    ApplicationSingleComputationPartition,
    Deterministic
);
region_totals!(
    UnboundedBytesTotals,
    RegionKey,
    DeterministicPartitioned,
    bytes usize::MAX
);
region_totals!(ProbedTotals, probe::ProbeKey, DeterministicPartitioned);

/// Which computation a region totals program declares, and which owner binding
/// its contribution installs for it.
trait RegionTotalsBinding: 'static {
    type Computation: ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>;

    /// Installs the computation's owner and returns the handler of the region
    /// totals demand, which runs the computation when its owner can.
    fn install(setup: &mut owner::Setup<'_>) -> owner::Installed;
}

/// The entry facts, the demand that totals them, the region output's
/// operation and the edit of one entry fact, for the topology's schema.
pub(crate) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    entry_correction::declare(entry_edit::declare(region_output::declare(
        demand::declare(schema),
    )))
}

/// The topology's own handlers: the demand and the region output have no
/// owner to run.
pub(crate) fn configure<Schema: TopologySchemaBinding>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<demand::RegionTotalsDemandBinding<Schema>, _>(
        demand::RegionTotalsHandler::idle(),
    )?;
    idle_entry_handlers(setup)
}

/// The region output's handler with no owner to run, and the entry edit's.
fn idle_entry_handlers<Schema: TopologySchemaBinding>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<region_output::RegionOutputBinding<Schema>, _>(
        region_output::RegionOutputHandler::idle(),
    )?;
    setup.handler::<entry_edit::EntryEditBinding<Schema>, _>(entry_edit::EntryEditHandler)?;
    entry_correction::configure(setup)
}

/// A program that only declares the computation. Nothing installs its owner,
/// so the program is authored and validated and never installed.
struct DeclaredOnly<Computation>(PhantomData<fn() -> Computation>);
impl<Computation> RegionTotalsBinding for DeclaredOnly<Computation>
where
    Computation: ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>,
{
    type Computation = Computation;

    fn install(_: &mut owner::Setup<'_>) -> owner::Installed {
        Ok(demand::RegionTotalsHandler::idle())
    }
}

/// The topology contribution plus the owner of the region totals computation.
struct RegionTotalsContribution<Binding>(PhantomData<fn() -> Binding>);
impl<Binding: RegionTotalsBinding> ApplicationSchemaContribution<CheckpointSchema>
    for RegionTotalsContribution<Binding>
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::IDENTITY;
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<CheckpointSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<CheckpointSchema> {
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::register_members(
            builder,
        )
    }
}
impl<Binding: RegionTotalsBinding> WorthQueryApplicationContribution<CheckpointSchema>
    for RegionTotalsContribution<Binding>
{
    type Configuration = TopologyConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <TopologyContribution as WorthQueryApplicationContribution<CheckpointSchema>>::contracts(
            contracts,
        )
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        let demand = Binding::install(setup)?;
        setup.handler::<demand::RegionTotalsDemandBinding<CheckpointSchema>, _>(demand)?;
        idle_entry_handlers(setup)?;
        TopologyContribution::configure_topology(configuration, setup)
    }
}

struct RegionTotalsProgram<Binding>(PhantomData<fn() -> Binding>);
impl<Binding: RegionTotalsBinding> ApplicationProgramDefinition<CheckpointSchema>
    for RegionTotalsProgram<Binding>
{
    type Contributions = (RegionTotalsContribution<Binding>,);
    type Outputs = ApplicationProgramOutputs<CheckpointRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-region-totals-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            demand_policy::final_output_feature::<demand_policy::FinalArtifact>()
                .managed_computation::<Binding::Computation>()
                .mutation::<demand::RegionTotalsDemandBinding<CheckpointSchema>>()
                .finish(),
        )
    }
}

fn revision_of<Computation>() -> ApplicationProgramRevision
where
    Computation: ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>,
{
    *ApplicationProgramAuthoring::<
        CheckpointSchema,
        RegionTotalsProgram<DeclaredOnly<Computation>>,
    >::begin()
    .validated_program()
    .expect("the region totals program is complete")
    .revision()
}

fn denial_of<Computation>() -> Option<ApplicationProgramValidationDenialKind>
where
    Computation: ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>,
{
    ApplicationProgramAuthoring::<
        CheckpointSchema,
        RegionTotalsProgram<DeclaredOnly<Computation>>,
    >::begin()
    .validated_program()
    .err()
    .map(|denial| denial.kind())
}

#[test]
fn program_with_one_partitioned_computation_installs_with_its_owner() {
    let _guard = checkpoint_recovery_test_guard();
    let program = ApplicationProgramAuthoring::<
        CheckpointSchema,
        RegionTotalsProgram<owner::PartitionedOwner>,
    >::begin()
    .validated_program()
    .expect("the region totals program is complete");
    let computations = program
        .features()
        .iter()
        .flat_map(|feature| feature.managed_computations())
        .collect::<Vec<_>>();
    assert_eq!(computations.len(), 1);
    assert_eq!(
        computations[0].execution(),
        ApplicationComputationExecution::DeterministicPartitioned
    );
    assert_eq!(computations[0].partition(), RegionKey::IDENTITY);
    assert_eq!(
        computations[0].determinism(),
        DeterminismContract::CanonicalBitwise
    );

    let application = support::install_program::<RegionTotalsProgram<owner::PartitionedOwner>>(
        None,
        Default::default(),
    );
    let (scope, principal) = authenticate(&application);
    application
        .request(&principal, &scope)
        .retain_read()
        .expect("the installed program serves a read");
}

#[test]
fn computation_partition_key_identity_is_program_revision_meaning() {
    assert_eq!(revision_of::<RegionTotals>(), revision_of::<RegionTotals>());
    assert_ne!(
        revision_of::<RegionTotals>(),
        revision_of::<RenamedKeyTotals>()
    );
}

#[test]
fn determinism_contract_is_program_revision_meaning() {
    assert_ne!(
        revision_of::<RegionTotals>(),
        revision_of::<EquivalentTotals>()
    );
}

#[test]
fn partitioned_computation_may_not_declare_the_single_partition() {
    assert_eq!(denial_of::<RegionTotals>(), None);
    assert_eq!(
        denial_of::<SingleKeyPartitionedTotals>(),
        Some(ApplicationProgramValidationDenialKind::MismatchedManagedComputationPartition)
    );
}

#[test]
fn deterministic_computation_may_not_declare_its_own_partition_key() {
    assert_eq!(
        denial_of::<KeyedSerialTotals>(),
        Some(ApplicationProgramValidationDenialKind::MismatchedManagedComputationPartition)
    );
}
