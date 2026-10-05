//! A partitioned managed computation declared and installed through the public
//! facade: its computation partition key and determinism contract are program
//! meaning, and its execution posture decides which key it may declare.

use std::collections::BTreeMap;
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
    WorthQueryApplicationContributionSetup, WorthQueryManagedComputationCheckpoint,
    WorthQueryManagedComputationDenial, WorthQueryManagedComputationOwner,
    WorthQueryManagedComputationPrepared,
};

use super::*;

/// Body lengths tagged with the region each body lies in.
struct RegionLengths;
impl ApplicationComputationInput for RegionLengths {
    type Value = Vec<(u32, u64)>;
    const IDENTITY: &'static str = "checkpoint-region-lengths";
}

macro_rules! region_key {
    ($key:ident, $identity:literal) => {
        #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        struct $key(u32);
        impl From<u32> for $key {
            fn from(region: u32) -> Self {
                Self(region)
            }
        }
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
    ($computation:ident, $partition:ty, $execution:ident $(, $determinism:expr)?) => {
        struct $computation;
        impl ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>
            for $computation
        {
            type Input = RegionLengths;
            type Output = demand_policy::FinalArtifact;
            type Partition = $partition;
            type Reuse = NoWarmStart;
            type Stopped = RegionStopped;
            const IDENTITY: &'static str = "checkpoint-region-totals";
            const EXECUTION: ApplicationComputationExecution =
                ApplicationComputationExecution::$execution;
            $(const DETERMINISM: DeterminismContract = $determinism;)?
            const RESOURCES: ApplicationComputationResourceCeiling =
                ApplicationComputationResourceCeiling::new(4_096, 8_192);
        }
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

struct LengthsByRegion<Key>(BTreeMap<Key, Vec<u64>>);
impl<Key: Send + 'static> WorthQueryManagedComputationPrepared for LengthsByRegion<Key> {
    fn retained_bytes(&self) -> usize {
        self.0.values().map(Vec::len).sum::<usize>() * std::mem::size_of::<u64>()
    }
}

struct RegionTotalsOwner;
impl<Computation>
    WorthQueryManagedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, Computation>
    for RegionTotalsOwner
where
    Computation: ApplicationManagedComputation<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Input = RegionLengths,
    >,
    Computation::Partition: From<u32>,
{
    type Prepared = LengthsByRegion<Computation::Partition>;
    type Computed = Vec<u64>;
    type Output = Vec<u64>;
    type Stopped = ();

    fn prepare(
        &self,
        lengths: &<RegionLengths as ApplicationComputationInput>::Value,
    ) -> Result<Self::Prepared, ()> {
        let mut regions = BTreeMap::<Computation::Partition, Vec<u64>>::new();
        for (region, length) in lengths {
            regions.entry((*region).into()).or_default().push(*length);
        }
        Ok(LengthsByRegion(regions))
    }

    fn compute(
        &self,
        prepared: &Self::Prepared,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Vec<u64>, WorthQueryManagedComputationDenial<()>> {
        let mut totals = Vec::with_capacity(prepared.0.len());
        for lengths in prepared.0.values() {
            checkpoint.advance(lengths.len())?;
            totals.push(lengths.iter().sum());
        }
        Ok(totals)
    }

    fn complete(&self, _: Self::Prepared, totals: Vec<u64>) -> Result<Vec<u64>, ()> {
        Ok(totals)
    }
}

/// The topology contribution plus the owner of the region totals computation.
struct RegionTotalsContribution;
impl ApplicationSchemaContribution<CheckpointSchema> for RegionTotalsContribution {
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
impl WorthQueryApplicationContribution<CheckpointSchema> for RegionTotalsContribution {
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
        <TopologyContribution as WorthQueryApplicationContribution<CheckpointSchema>>::configure(
            configuration,
            setup,
        )?;
        setup
            .computation::<PlanarFinalOutputFeature, RegionTotals, _>(RegionTotalsOwner)
            .map(drop)
    }
}

struct RegionTotalsProgram<Computation>(PhantomData<fn() -> Computation>);
impl<Computation> ApplicationProgramDefinition<CheckpointSchema>
    for RegionTotalsProgram<Computation>
where
    Computation: ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>,
{
    type Contributions = (RegionTotalsContribution,);
    type Outputs = ApplicationProgramOutputs<CheckpointRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-region-totals-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            demand_policy::final_output_feature::<demand_policy::FinalArtifact>()
                .managed_computation::<Computation>()
                .finish(),
        )
    }
}

fn revision_of<Computation>() -> ApplicationProgramRevision
where
    Computation: ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>,
{
    *ApplicationProgramAuthoring::<CheckpointSchema, RegionTotalsProgram<Computation>>::begin()
        .validated_program()
        .expect("the region totals program is complete")
        .revision()
}

fn denial_of<Computation>() -> Option<ApplicationProgramValidationDenialKind>
where
    Computation: ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>,
{
    ApplicationProgramAuthoring::<CheckpointSchema, RegionTotalsProgram<Computation>>::begin()
        .validated_program()
        .err()
        .map(|denial| denial.kind())
}

#[test]
fn program_with_one_partitioned_computation_installs_with_its_owner() {
    let _guard = checkpoint_recovery_test_guard();
    let program =
        ApplicationProgramAuthoring::<CheckpointSchema, RegionTotalsProgram<RegionTotals>>::begin()
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

    let application =
        support::install_program::<RegionTotalsProgram<RegionTotals>>(None, Default::default());
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
