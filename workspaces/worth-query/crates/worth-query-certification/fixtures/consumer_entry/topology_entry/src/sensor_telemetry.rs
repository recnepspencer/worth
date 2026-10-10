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
pub struct Reading {
    id: u64,
    station: u32,
    millivolts: u64,
}
impl ChargedBytes for Reading {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ApplicationComputationPartition for Reading {
    const IDENTITY: &'static str = "telemetry.reading.v1";
}

pub struct TelemetryInput;
impl ApplicationComputationInput for TelemetryInput {
    type Value = Vec<Reading>;
    const IDENTITY: &'static str = "telemetry.input.v1";
}
#[derive(Serialize)]
pub struct Station(u32);
impl ApplicationComputationPartition for Station {
    const IDENTITY: &'static str = "telemetry.station.v1";
}
pub struct TelemetryReuse;
impl ApplicationComputationReuse for TelemetryReuse {
    const IDENTITY: &'static str = "telemetry.reuse.v1";
}
pub struct TelemetryStopped;
impl ApplicationComputationStopped for TelemetryStopped {
    const IDENTITY: &'static str = "telemetry.stopped.v1";
}

pub struct PeakReading<A>(PhantomData<fn() -> A>);
impl<S, F, A> ApplicationManagedComputation<S, F> for PeakReading<A>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    type Input = TelemetryInput;
    type Output = A;
    type Partition = Station;
    type Reuse = TelemetryReuse;
    type Stopped = TelemetryStopped;
    const IDENTITY: &'static str = "telemetry.peak.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(100_000, 1_048_576);
}

pub struct PeakOwner<Op>(PhantomData<fn() -> Op>);
impl<S, F, A, Op> WorthQueryPartitionedComputationOwner<S, F, PeakReading<A>> for PeakOwner<Op>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    type Operation = Op;
    type Item = Reading;
    type Gathered = Vec<u64>;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;

    fn partitions(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        input: &Vec<Reading>,
    ) -> Result<WorthQueryComputationPartitionPlan<Reading>, WorthQueryComputationInputDenial<u32>>
    {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            input.iter().cloned(),
            |reading| PartitionItemId(reading.id),
        ))
    }

    fn partition_key(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<Reading>,
        item: &Reading,
    ) -> Result<Station, WorthQueryComputationInputDenial<u32>> {
        Ok(Station(item.station))
    }

    fn gather(
        &self,
        _reader: &mut WorthQueryComputationReader<'_, '_, '_, S, Op>,
        _input: &Vec<Reading>,
        partition: WorthQueryComputationPartitionMembers<'_, Station, Reading>,
    ) -> Result<Vec<u64>, WorthQueryComputationInputDenial<u32>> {
        Ok(partition
            .items()
            .map(|(_, reading)| reading.millivolts)
            .collect())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Station, Vec<u64>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        let mut peak = 0_u64;
        for millivolts in partition.gathered() {
            checkpoint.advance(1)?;
            peak = peak.max(*millivolts);
        }
        Ok(peak)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |left, right| (*left).max(*right))
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}

pub fn declare_peak<S, F, A>() -> ApplicationFeatureSpec
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
{
    ApplicationFeatureSpec::root::<S, F>()
        .derived_artifact::<A>()
        .managed_computation::<PeakReading<A>>()
        .finish()
}

pub fn install_peak<S, F, A, Op>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<
    WorthQueryInstalledPartitionedComputation<S, F, PeakReading<A>, PeakOwner<Op>>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    A: ApplicationDerivedArtifact<S, F>,
    Op: 'static,
{
    setup.partitioned_computation::<F, PeakReading<A>, _>(PeakOwner(PhantomData))
}

pub fn run_peak<S, F, A, Op, B>(
    installed: &WorthQueryInstalledPartitionedComputation<S, F, PeakReading<A>, PeakOwner<Op>>,
    reader: &mut DecisionReader<'_, '_, '_, S, B>,
    input: &Vec<Reading>,
) -> Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>
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
pub(crate) fn sample_input() -> Vec<Reading> {
    vec![
        Reading {
            id: 1,
            station: 10,
            millivolts: 220,
        },
        Reading {
            id: 2,
            station: 20,
            millivolts: 850,
        },
        Reading {
            id: 3,
            station: 10,
            millivolts: 400,
        },
    ]
}
