//! A second declared computation: same operations, different code identity.
use super::*;
pub(super) struct ChangedCodeTotals;
impl ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>
    for ChangedCodeTotals
{
    type Input = RegionEntries;
    type Output = RegionArtifact;
    type Partition = RegionKey;
    type Reuse = NoWarmStart;
    type Stopped = RegionStopped;
    const IDENTITY: &'static str = "checkpoint-region-output-totals-edited-code";
    const EXECUTION: ApplicationComputationExecution =
        <OracleTotals as ApplicationManagedComputation<
            CheckpointSchema,
            PlanarFinalOutputFeature,
        >>::EXECUTION;
    const RESOURCES: ApplicationComputationResourceCeiling =
        <OracleTotals as ApplicationManagedComputation<
            CheckpointSchema,
            PlanarFinalOutputFeature,
        >>::RESOURCES;
}
