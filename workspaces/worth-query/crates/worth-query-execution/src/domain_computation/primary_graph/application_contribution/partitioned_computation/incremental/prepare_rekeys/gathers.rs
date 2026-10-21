//! Merge changed groups with borrowed canonical ranges, without a flat roster.
use super::super::super::routing::ComputationPartitionRouting;
use super::super::super::{gather_memory::GatheredMemory, WorthQueryComputationPartitionMembers};
use super::super::retained::RetainedPartitions;
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::{
    CallShapeTransition, ComputationCallSequence,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::PartitionIdentity;

pub(super) fn prepare<S, F, C, O, K, I, R>(
    owner: &O,
    reader: &mut Reader<'_, '_, S, O::Operation>,
    execution: &QueryRequestExecution<'_>,
    input: &InputValue<S, F, C>,
    retained: &RetainedPartitions<K, I, R>,
    sequence: &Arc<ComputationCallSequence>,
    routing: &ComputationPartitionRouting,
    transition: &CallShapeTransition,
    affected: &BTreeSet<PartitionIdentity>,
    mut fresh_keys: BTreeMap<PartitionIdentity, (Arc<K>, u64)>,
    declared: u64,
    prepared: &mut PreparedIncremental<K, I, R, O::Gathered>,
) -> Result<(), Denial<S, F, C, O>>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    C: ApplicationManagedComputation<S, F, Partition = K>,
    K: ApplicationComputationPartition,
    I: ApplicationComputationPartition + ChargedBytes,
    O: WorthQueryPartitionedComputationOwner<S, F, C, Item = I, PartitionResult = R>,
{
    let mut cursor = sequence.prefix();
    for id in affected {
        let read = ComputationRead::Partition(*id);
        // The rank of a newly inserted call is bounded by its next old call.
        let next = sequence.shape().lower_bound(read);
        carry_or_gather(
            owner,
            reader,
            execution,
            input,
            retained,
            sequence,
            routing,
            transition,
            cursor..next,
            declared,
            prepared,
            &mut fresh_keys,
        )?;
        if routing.digest(*id).is_some() {
            gather(
                owner,
                reader,
                execution,
                input,
                retained,
                routing,
                *id,
                declared,
                prepared,
                &mut fresh_keys,
            )?;
        }
        cursor = next + usize::from(sequence.shape().ordinal(read).is_some());
    }
    carry_or_gather(
        owner,
        reader,
        execution,
        input,
        retained,
        sequence,
        routing,
        transition,
        cursor..sequence.shape().len(),
        declared,
        prepared,
        &mut fresh_keys,
    )
}
fn carry_or_gather<S, F, C, O, K, I, R>(
    owner: &O,
    reader: &mut Reader<'_, '_, S, O::Operation>,
    execution: &QueryRequestExecution<'_>,
    input: &InputValue<S, F, C>,
    retained: &RetainedPartitions<K, I, R>,
    sequence: &Arc<ComputationCallSequence>,
    routing: &ComputationPartitionRouting,
    transition: &CallShapeTransition,
    source: std::ops::Range<usize>,
    declared: u64,
    prepared: &mut PreparedIncremental<K, I, R, O::Gathered>,
    fresh_keys: &mut BTreeMap<PartitionIdentity, (Arc<K>, u64)>,
) -> Result<(), Denial<S, F, C, O>>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    C: ApplicationManagedComputation<S, F, Partition = K>,
    K: ApplicationComputationPartition,
    I: ApplicationComputationPartition + ChargedBytes,
    O: WorthQueryPartitionedComputationOwner<S, F, C, Item = I, PartitionResult = R>,
{
    sequence.visit(source, &BTreeSet::new(), &mut |visit| match visit {
        CallVisit::Range(source, summary) => {
            let first = sequence
                .shape()
                .call_at(source.start)
                .expect("nonempty measured range");
            let start = transition
                .target()
                .ordinal(first)
                .expect("unchanged range in target");
            reader
                .carry_mapped_call_range(
                    &COMPARATOR,
                    sequence,
                    source.clone(),
                    start..start + source.len(),
                    summary,
                    execution,
                )
                .map_err(D::Resource)
        }
        CallVisit::Leaf(_, ComputationRead::Partition(id), _, _) => {
            gather(
                owner, reader, execution, input, retained, routing, id, declared, prepared,
                fresh_keys,
            )?;
            Ok(true)
        }
        CallVisit::Leaf(..) => unreachable!("only gather suffix"),
    })
}
fn gather<S, F, C, O, K, I, R>(
    owner: &O,
    reader: &mut Reader<'_, '_, S, O::Operation>,
    execution: &QueryRequestExecution<'_>,
    input: &InputValue<S, F, C>,
    retained: &RetainedPartitions<K, I, R>,
    routing: &ComputationPartitionRouting,
    id: PartitionIdentity,
    declared: u64,
    prepared: &mut PreparedIncremental<K, I, R, O::Gathered>,
    fresh_keys: &mut BTreeMap<PartitionIdentity, (Arc<K>, u64)>,
) -> Result<(), Denial<S, F, C, O>>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    C: ApplicationManagedComputation<S, F, Partition = K>,
    K: ApplicationComputationPartition,
    I: ApplicationComputationPartition + ChargedBytes,
    O: WorthQueryPartitionedComputationOwner<S, F, C, Item = I, PartitionResult = R>,
{
    super::super::super::super::super::fact_verification_observation::computation_prepared_partition();
    let (key, key_bytes) = match retained.partitions.get(&id) {
        Some(prior) => (Arc::clone(&prior.key), prior.key_bytes),
        None => fresh_keys
            .remove(&id)
            .expect("new routed group has a freshly read key"),
    };
    let mut memory = GatheredMemory::new(execution, declared)?;
    memory.before_members(execution, routing.members(id).count())?;
    let members = routing.members(id).collect::<Arc<[_]>>();
    let (gathered, charge) = reader.measured(ComputationRead::Partition(id), |reader| {
        owner.gather(
            &mut WorthQueryComputationReader::lend(reader),
            input,
            WorthQueryComputationPartitionMembers::new(id, &key, &members, &retained.items),
        )
    });
    let gathered = gathered.map_err(|reason| D::gathering(id, reason))?;
    prepared.gathered(id, key, key_bytes, members, gathered, charge, memory)
}
