//! Full-only kind projection with numeric, intrinsically admitted incidence rows.
use crate::identity::data::{KindId, RelationId};
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::residency::RegionContentFailure as Failure;
use crate::storage::substrate::SharedMap;
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
            .map_err(|cause| Failure::Execution(cause.into()))?;
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
                .map_err(Failure::Execution)?;
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
                    .map_err(|cause| Failure::Execution(cause.into()))?;
                for (kind, relation) in adjacency.kind_memberships(basis) {
                    result.insert(relation, kind, incoming, slot, request)?;
                }
            }
        }
        Ok(result)
    }
}
