//! A carried walk stays independent from the added ordinary-field partition.
use super::*;
use worth_execution::PartitionItemId;
use worth_query_decl::facade::application_program::*;
pub type Root = WorthQueryInvariantEntityIdentity<Schema, Node>;
pub struct Input;
impl ApplicationComputationInput for Input {
    type Value = Root;
    const IDENTITY: &'static str = "succession.walk.input";
}
#[derive(Clone, serde::Serialize)]
pub struct Part(pub u64);
impl ApplicationComputationPartition for Part {
    const IDENTITY: &'static str = "succession.walk.part";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "succession.walk.root";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub struct Port;
impl ApplicationOutputPort<Schema, program::SplitParent> for Port {
    type Value = RowBinding;
    const IDENTITY: &'static str = "succession.walk.port";
}
pub struct Artifact;
impl ApplicationDerivedArtifact<Schema, program::SplitParent> for Artifact {
    type Output = Port;
    type Locality = Locality;
    const IDENTITY: &'static str = "succession.walk.artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = false;
    const PRODUCER_FAMILY: &'static str = "split-parent";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <source_control::Binding as ApplicationMutationBinding<Schema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "same-observed-inputs";
    const STOPPED_OUTCOME: &'static str = "walk-refused";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "succession.walk.reuse";
}
pub struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "succession.walk.stop";
}
pub struct Computation;
impl ApplicationManagedComputation<Schema, program::SplitParent> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Part;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "succession.walk.computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
pub struct Owner(pub Arc<Calls>);
pub type Installed =
    WorthQueryInstalledPartitionedComputation<Schema, program::SplitParent, Computation, Owner>;
type Reader<'a, 'b, 'c> =
    WorthQueryComputationReader<'a, 'b, 'c, Schema, producers::parent::preserve::Operation>;
impl WorthQueryPartitionedComputationOwner<Schema, program::SplitParent, Computation> for Owner {
    type Operation = producers::parent::preserve::Operation;
    type Item = Part;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;
    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        root: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<Part>, WorthQueryComputationInputDenial<u32>>
    {
        let width = reader.field(root, Width::at("Width"))?.unwrap();
        Ok(WorthQueryComputationPartitionPlan::keyed(
            (0..(1 + u64::from(width >= 2))).map(Part),
            |p| PartitionItemId(p.0),
        ))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
        item: &Part,
    ) -> Result<Part, WorthQueryComputationInputDenial<u32>> {
        Ok(item.clone())
    }
    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        root: &Root,
        members: WorthQueryComputationPartitionMembers<'_, Part, Part>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        self.0
            .gathers
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if members.key().0 == 0 {
            Ok(reader
                .walk(GeneratedBy::reference(), RelationDirection::Forward, root)?
                .len() as u64)
        } else {
            // A fresh owner call must not rescue missing issuance for the carried walk.
            Ok(reader.field(root, Value::at("Value"))?.unwrap())
        }
    }
    fn compute_partition(
        &self,
        part: WorthQueryComputationPartitionView<'_, Part, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        checkpoint.poll()?;
        Ok(*part.gathered())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |a, b| a + b)
    }
    fn complete(&self, value: u64) -> Result<u64, u32> {
        self.0.outcomes.lock().unwrap().push(value);
        Ok(value)
    }
}
