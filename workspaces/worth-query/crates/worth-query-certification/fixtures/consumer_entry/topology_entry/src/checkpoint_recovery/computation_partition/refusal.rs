//! Installation refuses an owner binding that does not serve the computation's
//! declared execution, and a determinism contract it cannot honor.

use worth_query_host::facade::application_contribution::{
    PartitionItemId, WorthQueryComputationPartitionPlan, WorthQueryComputationPartitionView,
    WorthQueryDeterministicReducer, WorthQueryManagedComputationCheckpoint,
    WorthQueryManagedComputationDenial, WorthQueryManagedComputationOwner,
    WorthQueryManagedComputationPrepared, WorthQueryPartitionedComputationOwner,
};
use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind;

use super::owner::{Entries, Installed, RegionTotalsHandler, RegionTotalsOwner, Setup};
use super::*;

/// The single-partition owner: it totals the whole input in one computation.
struct WholeInputOwner;
struct WholeInput(Vec<f64>);
impl WorthQueryManagedComputationPrepared for WholeInput {
    fn retained_bytes(&self) -> usize {
        self.0.len() * std::mem::size_of::<f64>()
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

    fn prepare(&self, entries: &Entries) -> Result<WholeInput, ()> {
        Ok(WholeInput(
            entries.iter().map(|entry| entry.value).collect(),
        ))
    }

    fn compute(
        &self,
        prepared: &WholeInput,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<()>> {
        checkpoint.advance(prepared.0.len())?;
        Ok(prepared.0.iter().sum())
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
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        entries: &Entries,
    ) -> WorthQueryComputationPartitionPlan<ApplicationSingleComputationPartition> {
        WorthQueryComputationPartitionPlan::keyed(
            entries,
            |entry| PartitionItemId(entry.id),
            |_| ApplicationSingleComputationPartition,
        )
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<
            '_,
            ApplicationSingleComputationPartition,
            Entries,
        >,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        checkpoint.advance(partition.items().len())?;
        Ok(partition
            .items()
            .iter()
            .map(|item| partition.input()[item.position()].value)
            .sum())
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
