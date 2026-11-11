//! Request-funded admission of the selected Head's kind meaning.
use super::{funding, Failure};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::{AdjacencyKindBasis, SparseAdjacencyTable};
use crate::storage::residency::region_image::RegionAdjacencyKinds;
use worth_execution::ExecutionRequest;

pub(super) fn verify_buckets(
    partition: &PartitionState,
    outgoing: &[RegionAdjacencyKinds],
    incoming: &[RegionAdjacencyKinds],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    compare(&partition.adjacency, outgoing, request)?;
    compare(&partition.reverse_adjacency, incoming, request)
}

fn compare(
    table: &SparseAdjacencyTable,
    expected: &[RegionAdjacencyKinds],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let mut rows = expected.iter();
    for (&slot, adjacency) in table.iter() {
        funding::work(request, 1)?;
        let row = rows
            .next()
            .ok_or_else(|| unavailable(Cause::IntegrityMismatch))?;
        if row.slot != slot as u64 {
            return Err(unavailable(Cause::IntegrityMismatch));
        }
        let mut actual = adjacency.kind_memberships(AdjacencyKindBasis::Current);
        let mut previous = None;
        for (kind, relations) in &row.kinds {
            funding::work(request, 1)?;
            if relations.is_empty() || previous.is_some_and(|old| old >= *kind) {
                return Err(unavailable(Cause::IntegrityMismatch));
            }
            previous = Some(*kind);
            for &relation in relations {
                funding::work(request, 1)?;
                if actual.next() != Some((*kind, relation)) {
                    return Err(unavailable(Cause::IntegrityMismatch));
                }
            }
        }
        funding::work(request, 1)?;
        if actual.next().is_some() {
            return Err(unavailable(Cause::IntegrityMismatch));
        }
    }
    if rows.next().is_some() {
        return Err(unavailable(Cause::IntegrityMismatch));
    }
    Ok(())
}

fn unavailable(cause: Cause) -> Failure {
    crate::storage::data::RelationalStorageUnavailable {
        subject: crate::storage::data::RelationalStorageSubject::Region(
            crate::identity::data::PartitionId::main(),
        ),
        cause,
    }
    .into()
}
