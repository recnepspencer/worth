//! Where the region output sits in the oracle's program: the final output
//! feature's artifact, which the region output's producer makes, and the
//! connection whose demands select it.

use worth_query_decl::facade::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationConnectionIdentity,
    ApplicationDerivedArtifact, ApplicationFeature, ApplicationOccurrenceConnectionBinding,
};
use worth_query_host::facade::application_contribution::WorthQueryProducerOutputFamily;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

use super::super::output_producer::RegionOutputFamily;
use super::*;

/// The work the totals may cost: every entry of the largest set, and the
/// input's digest.
pub(super) const TOTALS_WORK: usize = WORK_PER_ENTRY * LARGEST_SET + WORK_BESIDE_ENTRIES;
/// The bytes the totals keep.
pub(super) const TOTALS_RETAINED_BYTES: usize = 8_192;

/// The region output, as the final output feature's artifact in place of
/// the final output.
pub(super) struct RegionArtifact;
impl ApplicationDerivedArtifact<CheckpointSchema, PlanarFinalOutputFeature> for RegionArtifact {
    type Output = PlanarFinalBodyOutput;
    type Locality = demand_policy::PlanarLocality;
    const IDENTITY: &'static str = "checkpoint-region-output-artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession =
        ApplicationArtifactSuccession::PreserveWhenEquivalent;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str =
        <RegionOutputFamily as WorthQueryProducerOutputFamily<CheckpointSchema>>::IDENTITY;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <PlanarOutputFeature as ApplicationFeature<CheckpointSchema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "exact-source";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(TOTALS_WORK, DECISION_FACT_BUDGET * 512);
    const STOPPED_OUTCOME: &'static str = "region-output-resource-denied";
}

/// Selects the region output of the scope a planar output names.
pub(super) struct RegionOutputConnection;

impl ApplicationConnectionIdentity for RegionOutputConnection {
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-region-output.v1";
}

impl
    ApplicationOccurrenceConnectionBinding<
        CheckpointSchema,
        PlanarOutputFeature,
        PlanarFinalOutputFeature,
    > for RegionOutputConnection
{
}

impl WorthQueryApplicationDependentOutputConnection<CheckpointSchema> for RegionOutputConnection {
    type RootDemand = PlanarOutputDemand;
    type Discovery = PlanarOutputRead;
    type Demand = RegionOutputDemand;
    const IDENTITY: &'static str = "worth.query.certification.planar-output-to-region-output.v1";

    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(PlanarOutputRead {
            body_key: root.body_key().to_owned(),
        })
    }

    fn demands_from_discovery(
        discovery: &PlanarOutputReadResult,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(vec![RegionOutputDemand(discovery.body_key.clone())])
    }
}

pub(super) type RegionConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    RegionOutputConnection,
>;

/// The checkpoint outputs, with the region output in place of the final one.
pub(super) type OracleRoot = ApplicationOutputGraph<
    RootConnection,
    (
        ApplicationOutputEdge<RegionConnection, ApplicationOutputLeaf>,
        ApplicationOutputEdge<AlternateConnection, ApplicationOutputLeaf>,
    ),
>;

/// The same declared work in an operation that has no producer output slot.
pub(super) struct UnretainedTotals<const WORK: usize>;
impl<const WORK: usize> ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature>
    for UnretainedTotals<WORK>
{
    type Input = RegionEntries;
    type Output = RegionArtifact;
    type Partition = RegionKey;
    type Reuse = NoWarmStart;
    type Stopped = RegionStopped;
    const IDENTITY: &'static str = "checkpoint-unretained-oracle-totals";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(WORK, TOTALS_RETAINED_BYTES);
}
