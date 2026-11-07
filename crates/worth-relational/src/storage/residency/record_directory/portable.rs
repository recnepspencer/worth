//! Canonical cold membership facts; ordinary publications never build this image.
use super::{
    maintenance::DirectoryError,
    membership::{identity, key, kind},
    *,
};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::maintenance::ReconstructionLane;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortableRecordDirectory {
    pub(crate) entities: Vec<(KindId, EntityId)>,
    pub(crate) relations: Vec<(KindId, RelationId)>,
}
impl RegionRecordDirectory {
    pub(crate) fn portable(
        &self,
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<PortableRecordDirectory>, DirectoryError> {
        lane.recover(execution, |lane, work| self.portable_controlled(lane, work))
    }
    fn portable_controlled(
        &self,
        _lane: &ReconstructionLane<'_>,
        work: &mut crate::storage::substrate::maintenance::ReconstructionWork<'_, '_>,
    ) -> Result<Arc<PortableRecordDirectory>, DirectoryError> {
        if let Some(portable) = self.portable.get() {
            return Ok(portable.clone());
        }
        let portable = (|| {
            let mut entities = Vec::new();
            let mut relations = Vec::new();
            for encoded in self.entities.keys() {
                work.prepare(1, size_of::<(KindId, EntityId)>() as u64)?;
                entities.push((kind(encoded), identity(encoded, self.partition)));
            }
            for encoded in self.relations.keys() {
                work.prepare(1, size_of::<(KindId, RelationId)>() as u64)?;
                relations.push((kind(encoded), identity(encoded, self.partition)));
            }
            Ok::<_, DirectoryError>(Arc::new(PortableRecordDirectory {
                entities,
                relations,
            }))
        })()?;
        Ok(self.portable.get_or_init(|| portable).clone())
    }
    #[cfg(test)]
    pub(crate) fn portable_oracle(&self) -> Result<Arc<PortableRecordDirectory>, DirectoryError> {
        ReconstructionLane::inspect_oracle(|lane, work| self.portable_controlled(lane, work))
    }

    pub(crate) fn from_portable(
        lane: &ReconstructionLane<'_>,
        partition: PartitionId,
        portable: Arc<PortableRecordDirectory>,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<Arc<Self>, DirectoryError> {
        let total = portable
            .entities
            .len()
            .checked_add(portable.relations.len())
            .ok_or(DirectoryError::Corrupt)?;
        // The existing map and digest owners provide coexistence bounds. Cold
        // construction also retains its ordered keys and digest input vector.
        let map = crate::storage::substrate::SharedMap::<u128, ()>::new()
            .adoption_allocation_bound()
            .ok_or(DirectoryError::Corrupt)?;
        let (_, digest) = crate::storage::substrate::DigestIndex::default().edit_quote(0);
        let bytes = (total as u64)
            .checked_mul(
                map.checked_add(digest)
                    .and_then(|n| n.checked_add(64))
                    .ok_or(DirectoryError::Corrupt)?,
            )
            .ok_or(DirectoryError::Corrupt)?;
        prepare((total as u64).saturating_mul(128), bytes)?;
        let entities = keys(partition, &portable.entities)?;
        let relations = keys(partition, &portable.relations)?;
        Ok(Arc::new(Self {
            partition,
            entities: Membership::cold(lane, entities)?,
            relations: Membership::cold(lane, relations)?,
            portable: OnceLock::from(portable),
        }))
    }
    pub(crate) fn capture(
        lane: &ReconstructionLane<'_>,
        partition: &PartitionState,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<Arc<Self>, DirectoryError> {
        let mut entities = Vec::new();
        let mut relations = Vec::new();
        for slot in partition.entity_arena.live_bitset.set_slots() {
            prepare(1, std::mem::size_of::<(KindId, EntityId)>() as u64)?;
            let record = partition
                .entity_arena
                .get_slot(slot)
                .ok_or(DirectoryError::Corrupt)?;
            entities.push((
                record.kind_id().ok_or(DirectoryError::Corrupt)?,
                EntityId::new(partition.partition_id, slot as u64, record.generation()),
            ));
        }
        for slot in partition.relation_arena.live_bitset.set_slots() {
            prepare(1, std::mem::size_of::<(KindId, RelationId)>() as u64)?;
            let record = partition
                .relation_arena
                .get_slot(slot)
                .ok_or(DirectoryError::Corrupt)?;
            relations.push((
                record.kind_id().ok_or(DirectoryError::Corrupt)?,
                RelationId::new(partition.partition_id, slot as u64, record.generation()),
            ));
        }
        entities.sort_unstable_by_key(|(kind, id)| key(*kind, *id));
        relations.sort_unstable_by_key(|(kind, id)| key(*kind, *id));
        Self::from_portable(
            lane,
            partition.partition_id,
            Arc::new(PortableRecordDirectory {
                entities,
                relations,
            }),
            prepare,
        )
    }
}
fn keys<K: Copy>(
    partition: PartitionId,
    entries: &[(KindId, crate::identity::data::RecordId<K>)],
) -> Result<Vec<u128>, DirectoryError> {
    let mut previous = None;
    entries
        .iter()
        .map(|(kind, id)| {
            let encoded = key(*kind, *id);
            if id.partition_id != partition || previous.is_some_and(|previous| previous >= encoded)
            {
                return Err(DirectoryError::Corrupt);
            }
            previous = Some(encoded);
            Ok(encoded)
        })
        .collect()
}

impl RegionRecordDirectory {
    pub(crate) fn verify_member(
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: &PortableRecordDirectory,
        partition: &PartitionState,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        let actual = lane
            .recover(execution, |lane, work| {
                Self::capture(lane, partition, &mut |units, bytes| {
                    work.prepare(units, bytes).map_err(Into::into)
                })
            })
            .map_err(|error: DirectoryError| error.into_failure(partition.partition_id))?;
        if actual
            .portable(lane, execution)
            .map_err(|error| error.into_failure(partition.partition_id))?
            .as_ref()
            != expected
        {
            return Err(DirectoryError::Corrupt.into_failure(partition.partition_id));
        }
        Ok(())
    }
}
