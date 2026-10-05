//! Installation refuses an owner binding that does not serve the computation's
//! declared execution, and a determinism contract it cannot honor.

use worth_query_host::facade::application_contribution::{
    WorthQueryComputationPartitionMembers, WorthQueryComputationPartitionPlan,
    WorthQueryComputationPartitionView, WorthQueryDeterministicReducer,
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial,
    WorthQueryManagedComputationOwner, WorthQueryManagedComputationPrepared,
    WorthQueryPartitionedComputationOwner,
};
use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind;

use super::demand::{RegionTotalsHandler, TotalRegions};
use super::facts::{self, Entry, EntryData, InputDenial, Reader, Set};
use super::owner::{Installed, RegionTotalsOwner, Setup};
use super::*;

/// The single-partition owner. The serial binding lends its owner no reader,
/// so this owner cannot read the set's entries: it is installed, to show which
/// binding a declaration admits, and never run.
struct WholeInputOwner;
struct WholeInput;
impl WorthQueryManagedComputationPrepared for WholeInput {
    fn retained_bytes(&self) -> usize {
        0
    }
}
impl<Computation>
    WorthQueryManagedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, Computation>
    for WholeInputOwner
where
    Computation: ApplicationManagedComputation<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Input = RegionEntries,
    >,
{
    type Prepared = WholeInput;
    type Computed = f64;
    type Output = f64;
    type Stopped = ();

    fn prepare(&self, _: &Set) -> Result<WholeInput, ()> {
        Ok(WholeInput)
    }

    fn compute(
        &self,
        _: &WholeInput,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<()>> {
        checkpoint.advance(1)?;
        Ok(-0.0)
    }

    fn complete(&self, _: WholeInput, total: f64) -> Result<f64, ()> {
        Ok(total)
    }
}

/// A partitioned owner for the one partition every `Deterministic` computation
/// has.
struct OnePartitionOwner;
impl WorthQueryPartitionedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, SerialTotals>
    for OnePartitionOwner
{
    type Operation = TotalRegions;
    type Item = Entry;
    type Gathered = Vec<EntryData>;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        set: &Set,
    ) -> Result<WorthQueryComputationPartitionPlan<Entry>, InputDenial> {
        Ok(facts::entries(reader, set)?)
    }

    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Set,
        _: &Entry,
    ) -> Result<ApplicationSingleComputationPartition, InputDenial> {
        Ok(ApplicationSingleComputationPartition)
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        _: &Set,
        partition: WorthQueryComputationPartitionMembers<
            '_,
            ApplicationSingleComputationPartition,
            Entry,
        >,
    ) -> Result<Vec<EntryData>, InputDenial> {
        Ok(facts::gathered(reader, partition.items())?)
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<
            '_,
            ApplicationSingleComputationPartition,
            Vec<EntryData>,
        >,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        checkpoint.advance(partition.items().len())?;
        Ok(partition.gathered().iter().map(|entry| entry.value).sum())
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(|| -0.0, |left, right| left + right)
    }

    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}

/// A `Deterministic` computation bound to the single-partition owner: the
/// binding every such computation already has.
struct SerialOwner;
impl RegionTotalsBinding for SerialOwner {
    type Computation = SerialTotals;
    fn install(setup: &mut Setup<'_>) -> Installed {
        setup.computation::<PlanarFinalOutputFeature, SerialTotals, _>(WholeInputOwner)?;
        Ok(RegionTotalsHandler::idle())
    }
}

struct SerialOwnerOfPartitioned;
impl RegionTotalsBinding for SerialOwnerOfPartitioned {
    type Computation = RegionTotals;
    fn install(setup: &mut Setup<'_>) -> Installed {
        setup.computation::<PlanarFinalOutputFeature, RegionTotals, _>(WholeInputOwner)?;
        Ok(RegionTotalsHandler::idle())
    }
}

struct PartitionedOwnerOfSerial;
impl RegionTotalsBinding for PartitionedOwnerOfSerial {
    type Computation = SerialTotals;
    fn install(setup: &mut Setup<'_>) -> Installed {
        setup.partitioned_computation::<PlanarFinalOutputFeature, SerialTotals, _>(
            OnePartitionOwner,
        )?;
        Ok(RegionTotalsHandler::idle())
    }
}

struct PartitionedOwnerOfEquivalent;
impl RegionTotalsBinding for PartitionedOwnerOfEquivalent {
    type Computation = EquivalentTotals;
    fn install(setup: &mut Setup<'_>) -> Installed {
        setup.partitioned_computation::<PlanarFinalOutputFeature, EquivalentTotals, _>(
            RegionTotalsOwner,
        )?;
        Ok(RegionTotalsHandler::idle())
    }
}

/// The denial installation gives the binding, or `None` when it installs.
fn installation_refusal<Binding: RegionTotalsBinding>(
) -> Option<WorthQueryPrimaryGraphInstallationDenialKind> {
    let _guard = checkpoint_recovery_test_guard();
    let installed = support::try_install_program_with_limits::<RegionTotalsProgram<Binding>>(
        None,
        Default::default(),
        support::limits(
            32,
            support::invalidation(128 * 1_024 * 1_024, 1_000_000, 128),
        ),
        support::seed_cycle,
    );
    match installed.err().map(|denial| *denial) {
        None => None,
        Some(WorthQueryInMemoryApplicationDenial::Contributions(denial)) => Some(denial.kind()),
        Some(other) => panic!("the contribution's own setup decides the binding: {other:?}"),
    }
}

#[test]
fn deterministic_computation_keeps_its_single_partition_owner_binding() {
    assert_eq!(installation_refusal::<SerialOwner>(), None);
}

#[test]
fn partitioned_computation_refuses_the_single_partition_owner_binding() {
    assert_eq!(
        installation_refusal::<SerialOwnerOfPartitioned>(),
        Some(WorthQueryPrimaryGraphInstallationDenialKind::ManagedComputationOwnerBindingMismatch)
    );
}

#[test]
fn deterministic_computation_refuses_the_partitioned_owner_binding() {
    assert_eq!(
        installation_refusal::<PartitionedOwnerOfSerial>(),
        Some(WorthQueryPrimaryGraphInstallationDenialKind::ManagedComputationOwnerBindingMismatch)
    );
}

#[test]
fn declared_equivalence_predicate_is_refused_at_installation() {
    assert_eq!(
        installation_refusal::<owner::PartitionedOwner>(),
        None,
        "the same owner and key install under the bitwise contract"
    );
    assert_eq!(
        installation_refusal::<PartitionedOwnerOfEquivalent>(),
        Some(
            WorthQueryPrimaryGraphInstallationDenialKind::ManagedComputationEquivalenceUnavailable
        )
    );
}
