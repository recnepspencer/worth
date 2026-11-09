//! Whole checkpoint memberships require a cold lane and original admission.
use super::{AdjacencySet, KindId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::substrate::maintenance::ReconstructionLane;

type Checkpoint<'a> = dyn FnMut(u64, u64) -> Result<(), Cause> + 'a;

impl AdjacencySet {
    pub(crate) fn checkpoint_kind_memberships(
        &self,
        _lane: &ReconstructionLane<'_>,
        basis: AdjacencyKindBasis,
        checkpoint: &mut Checkpoint<'_>,
    ) -> Result<Vec<(KindId, Vec<RelationId>)>, Cause> {
        let entries = self.entries();
        let selected = match basis {
            AdjacencyKindBasis::Historical => &entries.historical_by_kind,
            AdjacencyKindBasis::Current => &entries.current_by_kind,
        };
        let Some(kinds) = selected else {
            return Ok(Vec::new());
        };
        let mut answer = admitted_vec(kinds.len(), checkpoint)?;
        for (&kind, ids) in kinds {
            checkpoint(1, 0)?;
            let mut members = admitted_vec(ids.len(), checkpoint)?;
            for &relation in ids.keys() {
                checkpoint(1, 0)?;
                members.push(relation);
            }
            answer.push((kind, members));
        }
        Ok(answer)
    }
}

fn admitted_vec<T>(count: usize, checkpoint: &mut Checkpoint<'_>) -> Result<Vec<T>, Cause> {
    let bytes = count
        .checked_mul(size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Cause::MemoryExhausted)?;
    checkpoint(0, bytes)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| Cause::MemoryExhausted)?;
    Ok(values)
}
