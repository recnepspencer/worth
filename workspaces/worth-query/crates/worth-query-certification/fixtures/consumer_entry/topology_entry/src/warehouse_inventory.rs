// compiled-example: begin
use serde::Serialize;
use std::marker::PhantomData;
use worth_query_host::facade::{
    application_contribution::*,
    declaration::{
        application_operation::ApplicationMutationBinding, application_program::*,
        application_schema::ApplicationSchema,
    },
    primary_graph::{DecisionReader, WorthQueryPrimaryGraphInstallationDenial},
};

#[derive(Clone, Serialize)]
pub struct InventoryLine {
    id: u64,
    zone: u64,
    quantity: f64,
}
impl ChargedBytes for InventoryLine {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ApplicationComputationPartition for InventoryLine {
    const IDENTITY: &'static str = "inventory.line.v1";
}

pub struct InventoryInput;
impl ApplicationComputationInput for InventoryInput {
    type Value = Vec<InventoryLine>;
    const IDENTITY: &'static str = "inventory.input.v1";
}
#[derive(Serialize)]
pub struct Zone(u64);
impl ApplicationComputationPartition for Zone {
    const IDENTITY: &'static str = "inventory.zone.v1";
}
pub struct InventoryReuse;
impl ApplicationComputationReuse for InventoryReuse {
    const IDENTITY: &'static str = "inventory.reuse.v1";
}
pub struct InventoryStopped;
impl ApplicationComputationStopped for InventoryStopped {
    const IDENTITY: &'static str = "inventory.stopped.v1";
}

pub struct InventoryTotal<A>(PhantomData<fn() -> A>);
impl<S, F, A> ApplicationManagedComputation<S, F> for InventoryTotal<A>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    type Input = InventoryInput;
    type Output = A;
    type Partition = Zone;
    type Reuse = InventoryReuse;
    type Stopped = InventoryStopped;
    const IDENTITY: &'static str = "inventory.total.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(100_000, 1_048_576);
    // DETERMINISM defaults to canonical bitwise equality.
}

pub struct InventoryOwner<Op>(PhantomData<fn() -> Op>);
impl<S, F, A, Op> WorthQueryPartitionedComputationOwner<S, F, InventoryTotal<A>>
    for InventoryOwner<Op>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    type Operation = Op;
    type Item = InventoryLine;
    type Gathered = Vec<f64>;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;

    fn partitions(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        input: &Vec<InventoryLine>,
    ) -> Result<
        WorthQueryComputationPartitionPlan<InventoryLine>,
        WorthQueryComputationInputDenial<u32>,
    > {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            input.iter().cloned(),
            |line| PartitionItemId(line.id),
        ))
    }

    fn partition_key(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<InventoryLine>,
        item: &InventoryLine,
    ) -> Result<Zone, WorthQueryComputationInputDenial<u32>> {
        Ok(Zone(item.zone))
    }

    fn gather(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<InventoryLine>,
        partition: WorthQueryComputationPartitionMembers<'_, Zone, InventoryLine>,
    ) -> Result<Vec<f64>, WorthQueryComputationInputDenial<u32>> {
        Ok(partition.items().map(|(_, line)| line.quantity).collect())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Zone, Vec<f64>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        let mut total = 0.0;
        for quantity in partition.gathered() {
            checkpoint.advance(1)?;
            total += quantity;
        }
        Ok(total)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(|| 0.0, |left, right| left + right)
    }

    fn complete(&self, reduced: f64) -> Result<f64, u32> {
        Ok(reduced)
    }
}

pub fn declare_inventory<S, F, A>() -> ApplicationFeatureSpec
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    ApplicationFeatureSpec::root::<S, F>()
        .derived_artifact::<A>()
        .managed_computation::<InventoryTotal<A>>()
        .finish()
}

pub fn install_inventory<S, F, A, Op>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<
    WorthQueryInstalledPartitionedComputation<S, F, InventoryTotal<A>, InventoryOwner<Op>>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    setup.partitioned_computation::<F, InventoryTotal<A>, _>(InventoryOwner(PhantomData))
}

pub fn run_inventory<S, F, A, Op, B>(
    installed: &WorthQueryInstalledPartitionedComputation<
        S,
        F,
        InventoryTotal<A>,
        InventoryOwner<Op>,
    >,
    reader: &mut DecisionReader<'_, '_, '_, S, B>,
    input: &Vec<InventoryLine>,
) -> Result<(f64, u64), WorthQueryPartitionedComputationDenial<u32>>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
    B: ApplicationMutationBinding<S, Operation = Op>,
{
    let computed = installed
        .prepare(reader, input)?
        .compute(reader.managed_computation_execution())?;
    let charged_work = computed.charged_work();
    Ok((computed.complete()?, charged_work))
}
// compiled-example: end

#[cfg(test)]
pub(crate) fn sample_input() -> Vec<InventoryLine> {
    vec![
        InventoryLine {
            id: 1,
            zone: 10,
            quantity: 2.5,
        },
        InventoryLine {
            id: 2,
            zone: 20,
            quantity: 7.0,
        },
        InventoryLine {
            id: 3,
            zone: 10,
            quantity: 3.5,
        },
    ]
}
