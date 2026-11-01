//! Full-only normalization with numeric, intrinsically admitted incidence rows.
use crate::identity::data::{KindId, RelationId};
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::residency::RegionContentFailure as Failure;
use crate::storage::substrate::{RelationEndpoints, SharedMap};
use worth_execution::ExecutionRequest;

type Incidence = (RelationId, bool, usize);

#[derive(Debug, Default)]
pub(in crate::storage::residency) struct FullKindProjection {
    rows: SharedMap<Incidence, u32>,
}

impl FullKindProjection {
    pub(in crate::storage::residency) fn rows(&self) -> &SharedMap<Incidence, u32> {
        &self.rows
    }

    fn insert(
        &mut self,
        relation: RelationId,
        kind: KindId,
        incoming: bool,
        slot: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        request
            .consult()
            .map_err(|cause| Failure::Request(cause.into()))?;
        if let Some((key, prior)) = self.rows.lower_bound(&(relation, false, 0))
            && key.0 == relation
            && *prior != kind.0
        {
            return Err(Failure::Adjacency);
        }
        let key = (relation, incoming, slot);
        if !self.rows.contains_key(&key) {
            self.rows
                .insert_for_request(key, kind.0, request)
                .map_err(Failure::Request)?;
        }
        Ok(())
    }

    pub(in crate::storage::residency) fn capture(
        partition: &PartitionState,
        basis: AdjacencyKindBasis,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Failure> {
        let mut result = Self::default();
        for (incoming, table) in [
            (false, &partition.adjacency),
            (true, &partition.reverse_adjacency),
        ] {
            for (&slot, adjacency) in table.iter() {
                request
                    .consult()
                    .map_err(|cause| Failure::Request(cause.into()))?;
                for (kind, relation) in adjacency.kind_memberships(basis) {
                    result.insert(relation, kind, incoming, slot, request)?;
                }
            }
        }
        Ok(result)
    }

    pub(in crate::storage::residency) fn normalize(
        partition: &mut PartitionState,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Failure> {
        // Validate ALL supplied kind claims before discarding owned history,
        // exactly as ordinary canonical_history does before its peer filter.
        let supplied = Self::capture(partition, AdjacencyKindBasis::Historical, request)?;
        let mut expected = Self::default();
        for (&(relation, incoming, slot), &kind) in supplied.rows.iter() {
            if relation.partition_id != partition.partition_id {
                expected.insert(relation, KindId(kind), incoming, slot, request)?;
            }
        }
        drop(supplied);
        for slot in partition.relation_arena.occupied_slots_iter() {
            request
                .consult()
                .map_err(|cause| Failure::Request(cause.into()))?;
            for metadata in partition
                .relation_arena
                .metadata_history_at(slot)
                .into_iter()
                .flatten()
            {
                expected.add_owned(
                    partition,
                    RelationId::new(partition.partition_id, slot as u64, metadata.generation),
                    metadata.kind_id,
                    &metadata.endpoints,
                    request,
                )?;
            }
            if let Some(view) = partition.relation_arena.get_slot(slot)
                && let (Some(kind), Some(endpoints)) =
                    (view.kind_id(), view.extra().endpoints.as_ref())
            {
                expected.add_owned(
                    partition,
                    RelationId::new(partition.partition_id, slot as u64, view.generation()),
                    kind,
                    endpoints,
                    request,
                )?;
            }
        }
        expected.install(partition, request)?;
        Ok(expected)
    }

    fn add_owned(
        &mut self,
        partition: &PartitionState,
        relation: RelationId,
        kind: KindId,
        endpoints: &RelationEndpoints,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        if let Some((key, prior)) = self.rows.lower_bound(&(relation, false, 0))
            && key.0 == relation
            && *prior != kind.0
        {
            return Err(Failure::Adjacency);
        }
        if endpoints.source.partition_id == partition.partition_id
            && partition
                .adjacency
                .get(endpoints.source.slot_index())
                .is_some()
        {
            self.insert(
                relation,
                kind,
                false,
                endpoints.source.slot_index(),
                request,
            )?;
        }
        if endpoints.target.partition_id == partition.partition_id
            && partition
                .reverse_adjacency
                .get(endpoints.target.slot_index())
                .is_some()
        {
            self.insert(relation, kind, true, endpoints.target.slot_index(), request)?;
        }
        Ok(())
    }

    fn install(
        &self,
        partition: &mut PartitionState,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        for (incoming, table) in [
            (false, &mut partition.adjacency),
            (true, &mut partition.reverse_adjacency),
        ] {
            let original = table.clone();
            for (&slot, _) in original.iter() {
                let adjacency = table
                    .get_mut_for_request(slot, request)
                    .map_err(Failure::Request)?
                    .ok_or(Failure::Adjacency)?;
                adjacency.reset_historical_kind_buckets();
            }
            for (&(relation, side, slot), &kind) in self.rows.iter() {
                if side != incoming {
                    continue;
                }
                let adjacency = table
                    .get_mut_for_request(slot, request)
                    .map_err(Failure::Request)?
                    .ok_or(Failure::Adjacency)?;
                adjacency
                    .index_historical_kind_for_request(KindId(kind), relation, request)
                    .map_err(Failure::Request)?;
            }
        }
        Ok(())
    }
}
