//! Completed measured calls: retain patched custody or explicitly rebuild in full.
use super::*;
pub(super) fn complete<Key, Item, Stopped>(
    next: &NextPartitioning<Item>,
    partitions: &super::super::partition_metadata::PartitionMetadata<Key>,
    patched: Option<Arc<ComputationCallSequence>>,
    measured: bool,
    execution: &QueryRequestExecution<'_>,
) -> Result<Option<Arc<ComputationCallSequence>>, WorthQueryPartitionedComputationDenial<Stopped>> {
    let Some(membership) = next.membership.as_ref().filter(|_| measured) else {
        return Ok(None);
    };
    if let Some(patched) = patched {
        return Ok(Some(patched));
    }
    ComputationCallSequence::build(
        || {
            std::iter::once((
                ComputationRead::Membership,
                &membership.charge,
                membership.declared_units,
            ))
            .chain(next.item_keys.iter().map(|(item, call)| {
                (
                    ComputationRead::ItemKey(*item),
                    &call.charge,
                    call.declared_units,
                )
            }))
            .chain(
                partitions
                    .iter()
                    .map(|(id, partition)| (ComputationRead::Partition(*id), &partition.gather, 0)),
            )
        },
        execution,
    )
    .map(Some)
    .map_err(WorthQueryPartitionedComputationDenial::from_call_sequence)
}
