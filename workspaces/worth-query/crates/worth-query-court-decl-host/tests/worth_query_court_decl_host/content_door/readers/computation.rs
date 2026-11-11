//! One partition gathers the actual content field through its lent reader.
use super::super::*;
pub struct Port;
impl ApplicationOutputPort<Schema, world::Feature> for Port {
    type Value = operation::ResultBinding;
    const IDENTITY: &'static str = "content-reader.port";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "content-reader.root";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<Schema, world::Feature> for Artifact {
    type Output = Port;
    type Locality = Locality;
    const IDENTITY: &'static str = "content-reader.artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = false;
    const PRODUCER_FAMILY: &'static str = "content-reader";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "exact-field";
    const STOPPED_OUTCOME: &'static str = "refused";
}
#[derive(serde::Serialize)]
pub struct ReadInput {
    pub entity: WorthQueryInvariantEntityIdentity<Schema, Record>,
    pub impostor: bool,
}
pub struct Input;
impl ApplicationComputationInput for Input {
    type Value = ReadInput;
    const IDENTITY: &'static str = "content-reader.input";
}
#[derive(Clone, serde::Serialize)]
pub struct Part;
impl ChargedBytes for Part {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ApplicationComputationPartition for Part {
    const IDENTITY: &'static str = "content-reader.part";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "content-reader.reuse";
}
pub struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "content-reader.stop";
}
pub struct Computation;
impl ApplicationManagedComputation<Schema, world::Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Part;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "content-reader.computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
pub struct Owner;
pub type Installed =
    WorthQueryInstalledPartitionedComputation<Schema, world::Feature, Computation, Owner>;
type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, Schema, operation::Change>;
type Identity = ReadInput;
impl WorthQueryPartitionedComputationOwner<Schema, world::Feature, Computation> for Owner {
    type Operation = operation::Change;
    type Item = Part;
    type Gathered = Option<AdmittedContent<codec::GridFormat>>;
    type PartitionResult = Vec<u8>;
    type Output = Option<(ContentRefId, Vec<u8>)>;
    type Stopped = ContentReadDenial;
    fn partitions(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Identity,
    ) -> Result<
        WorthQueryComputationPartitionPlan<Part>,
        WorthQueryComputationInputDenial<ContentReadDenial>,
    > {
        Ok(WorthQueryComputationPartitionPlan::keyed([Part], |_| {
            PartitionItemId(0)
        }))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Identity,
        _: &Part,
    ) -> Result<Part, WorthQueryComputationInputDenial<ContentReadDenial>> {
        Ok(Part)
    }
    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &Identity,
        _: WorthQueryComputationPartitionMembers<'_, Part, Part>,
    ) -> Result<Self::Gathered, WorthQueryComputationInputDenial<ContentReadDenial>> {
        if input.impostor {
            assert!(matches!(
                reader.content(
                    &input.entity,
                    content_field(Values::at("Values"), codec::Impostor)
                ),
                Err(ContentReadDenial::BindingContractMismatch)
            ));
            return Ok(None);
        }
        reader
            .content(&input.entity, samples())
            .map_err(WorthQueryComputationInputDenial::Owner)
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Part, Self::Gathered>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Vec<u8>, WorthQueryManagedComputationDenial<ContentReadDenial>> {
        checkpoint.poll()?;
        Ok(partition
            .gathered()
            .as_ref()
            .map_or_else(Vec::new, |content| {
                content
                    .identity()
                    .0
                    .iter()
                    .chain(content.payload())
                    .copied()
                    .collect()
            }))
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<Vec<u8>> {
        WorthQueryDeterministicReducer::canonical(Vec::new, |a, b| {
            a.iter().chain(b).copied().collect()
        })
    }
    fn complete(&self, reduced: Vec<u8>) -> Result<Self::Output, ContentReadDenial> {
        if reduced.is_empty() {
            return Ok(None);
        }
        let (identity, payload) = reduced.split_at(32);
        Ok(Some((
            ContentRefId(identity.try_into().unwrap()),
            payload.to_vec(),
        )))
    }
}
