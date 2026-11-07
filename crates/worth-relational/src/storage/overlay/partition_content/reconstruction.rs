//! Full construction is distinct from predecessor-required maintenance.
use super::*;
impl PartitionContentCommitment {
    pub(crate) fn capture(
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: &PartitionState,
        symbols: &StringInterner,
    ) -> Result<(Self, u64), PartitionContentDigestError> {
        lane.recover(execution, |lane, work| {
            for prefix in partition
                .entity_arena
                .history_prefixes
                .iter()
                .chain(partition.relation_arena.history_prefixes.iter())
            {
                work.prepare(1, 0)?;
                if prefix.is_some() {
                    return Err(PartitionContentDigestError::InvalidHistoryCoverage);
                }
            }
            Self::capture_seeded(lane, work, partition, symbols, Self::default())
        })
    }
    /// Prefix openings must already have been authenticated by Head admission.
    pub(crate) fn capture_layered(
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: &PartitionState,
        symbols: &StringInterner,
    ) -> Result<(Self, u64), PartitionContentDigestError> {
        lane.recover(execution, |lane, work| {
            Self::capture_seeded(lane, work, partition, symbols, Self::default())
        })
    }
    pub(super) fn capture_seeded(
        lane: &ReconstructionLane<'_>,
        work: &mut ReconstructionWork<'_, '_>,
        partition: &PartitionState,
        symbols: &StringInterner,
        mut seed: Self,
    ) -> Result<(Self, u64), PartitionContentDigestError> {
        let mut hashed = seed.entities.rebuild(
            lane,
            work,
            &partition.entity_arena,
            symbols,
            |extra, symbols, work| {
                admission::hash_controlled(
                    b"entity-extra",
                    &entity_extra_material(extra, symbols)?,
                    work,
                )
            },
        )?;
        hashed += seed.relations.rebuild(
            lane,
            work,
            &partition.relation_arena,
            symbols,
            |extra, _, work| admission::hash_controlled(b"relation-extra", extra, work),
        )?;
        hashed += seed.adjacency.rebuild(lane, work, &partition.adjacency)?;
        hashed += seed
            .reverse_adjacency
            .rebuild(lane, work, &partition.reverse_adjacency)?;
        Ok((seed, hashed))
    }
}

#[cfg(test)]
impl PartitionContentCommitment {
    pub(crate) fn capture_oracle(
        partition: &PartitionState,
        symbols: &StringInterner,
    ) -> Result<(Self, u64), PartitionContentDigestError> {
        if partition
            .entity_arena
            .history_prefixes
            .iter()
            .chain(partition.relation_arena.history_prefixes.iter())
            .any(Option::is_some)
        {
            return Err(PartitionContentDigestError::InvalidHistoryCoverage);
        }
        ReconstructionLane::inspect_oracle(|lane, work| {
            Self::capture_seeded(lane, work, partition, symbols, Self::default())
        })
    }
}
