//! Existing collection traits bind the closed source count, not a custom value decoder.
pub use super::super::operation::GraphDenial;
pub use super::*;
pub struct CountCollection;
pub struct CountComputation;
pub struct Contract;
pub struct Artifact;
pub struct Output;
worth_query_host::facade::worth_query_structured_value_binding!(pub CountValueBinding for u64 {
    identity:"derived.count.value.v1"
});
impl<L: LawPolicy> ApplicationOutputPort<GraphSchema<L>, GraphFeature> for Output {
    type Value = CountValueBinding;
    const IDENTITY: &'static str = "derived.count.output";
}
impl ApplicationLocalityScope for Contract {
    const IDENTITY: &'static str = "derived.count.locality";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Neighborhood;
}
impl<L: LawPolicy> ApplicationDerivedArtifact<GraphSchema<L>, GraphFeature> for Artifact {
    type Output = Output;
    type Locality = Contract;
    const IDENTITY: &'static str = "derived.count.artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "derived.count.producer";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "derived.count.original-physical-facts";
    const STOPPED_OUTCOME: &'static str = "derived.count.stopped";
}
pub struct RootInput<L: LawPolicy>(pub std::marker::PhantomData<fn() -> L>);
impl<L: LawPolicy> ApplicationComputationInput for RootInput<L> {
    type Value = WorthQueryInvariantEntityIdentity<GraphSchema<L>, Node>;
    const IDENTITY: &'static str = "derived.count.exact-root";
}
impl ApplicationComputationReuse for Contract {
    const IDENTITY: &'static str = "derived.count.original-physical-facts";
}
impl ApplicationComputationStopped for Contract {
    const IDENTITY: &'static str = "derived.count.stopped";
}
impl<L: LawPolicy> ApplicationManagedComputation<GraphSchema<L>, GraphFeature>
    for CountComputation
{
    type Input = RootInput<L>;
    type Output = Artifact;
    type Partition = ApplicationDerivedRelationCountPartition;
    type Reuse = Contract;
    type Stopped = Contract;
    const IDENTITY: &'static str = "derived.count.computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
impl ApplicationCollectionContributor for Contract {
    const IDENTITY: &'static str = "derived.count.source";
}
impl ApplicationCollectionGrouping for Contract {
    const IDENTITY: &'static str = "derived.count.root";
}
impl ApplicationCollectionMeasures for Contract {
    const IDENTITY: &'static str = "derived.count.endpoint-pairs";
}
impl ApplicationCollectionLineage for Contract {
    const IDENTITY: &'static str = "derived.count.lineage";
}
impl ApplicationCollectionCompleteness for Contract {
    type Value = u64;
    type Stopped = GraphDenial;
    type Lineage = Contract;
    const IDENTITY: &'static str = "derived.count.complete";
    fn check(_: &u64) -> Result<(), GraphDenial> {
        Ok(())
    }
}
impl<L: LawPolicy> ApplicationDerivedCollection<GraphSchema<L>, GraphFeature> for CountCollection {
    type Computation = CountComputation;
    type Completeness = Contract;
    type Contributor = Contract;
    type Grouping = Contract;
    type Measures = Contract;
    type Lineage = Contract;
    const IDENTITY: &'static str = "derived.count.collection";
    const APPLICABILITY: &'static str = "derived.count.exact-root";
    const INCOMPLETE: ApplicationCollectionIncompletePosture =
        ApplicationCollectionIncompletePosture::RetainPending;
    const INCREMENTAL_UPDATE: &'static str = "derived.count.physical-and-empty-adjacency";
}
