//! Current-state materialization quotes, polled before state/membership visits.
use super::*;
use crate::storage::partition::AdjacencySet;
use crate::symbols::data::Symbol;
use worth_foundational::facade::{
    AuthoritativeRecordAspectState, ContractValidatedAspectValueView as View,
    PortableRecordAspectStateEntry,
};
pub(in crate::storage::residency) fn current(
    partition: &PartitionState,
    contracts: &[worth_foundational::facade::AspectContract],
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    // Column/map cursors allocate bounded traversal stacks. Admit their
    // simultaneous backing before constructing any Native iterator.
    let _cursors = Funding::reserve(vector::<[usize; 2]>(16 * usize::BITS as usize)?, request)?;
    let mut q = Quote::bytes(vector::<[usize; 2]>(16 * usize::BITS as usize)?, 0);
    let e = &partition.entity_arena;
    let r = &partition.relation_arena;
    for state in e
        .metadata_history
        .iter()
        .flat_map(|rows| rows.iter())
        .map(|meta| meta.authoritative_aspect_state())
        .chain(
            r.metadata_history
                .iter()
                .flat_map(|rows| rows.iter())
                .map(|meta| meta.authoritative_aspect_state()),
        )
        .chain(
            e.extra
                .iter()
                .map(|extra| extra.authoritative_aspect_state()),
        )
        .chain(
            r.extra
                .iter()
                .map(|extra| extra.authoritative_aspect_state()),
        )
    {
        work(request, 1)?;
        q.wire = sum(q.wire, state_wire(state, request)?)?;
    }
    for rows in e.metadata_history.iter() {
        work(request, 1)?;
        q.wire = sum(q.wire, vector::<VersionedEntityMetadataImage>(rows.len())?)?;
    }
    for rows in r.metadata_history.iter() {
        work(request, 1)?;
        q.wire = sum(
            q.wire,
            vector::<VersionedRelationMetadataImage>(rows.len())?,
        )?;
    }
    // Every fixed wire column has one entry per physical row. Tuple padding
    // is conservative for the simultaneous separate scalar Vec backings.
    q.wire = sum(
        q.wire,
        crate::storage::residency::region_capture::complete::current_scalar_wire([
            e.generations.len(),
            r.generations.len(),
        ])?,
    )?;
    for rows in [&e.aspect_versions, &r.aspect_versions] {
        q.wire = sum(
            q.wire,
            vector::<BTreeMap<crate::symbols::data::Symbol, u64>>(rows.len())?,
        )?;
        for row in rows.iter() {
            work(request, 1)?;
            q.wire = sum(
                q.wire,
                tree::<crate::symbols::data::Symbol, u64>(row.len())?,
            )?;
        }
    }
    for rows in [&e.field_revisions, &r.field_revisions] {
        q.wire = sum(
            q.wire,
            vector::<
                Option<
                    BTreeMap<
                        (crate::symbols::data::Symbol, crate::symbols::data::Symbol),
                        crate::storage::data::RelationalFieldRevision,
                    >,
                >,
            >(rows.len())?,
        )?;
        for row in rows.iter().flatten() {
            work(request, 1)?;
            q.wire = sum(
                q.wire,
                tree::<
                    (crate::symbols::data::Symbol, crate::symbols::data::Symbol),
                    crate::storage::data::RelationalFieldRevision,
                >(row.len())?,
            )?;
        }
    }
    q.wire = sum(q.wire, diagnostics(partition, request)?)?;
    // Concrete owned wire column shells and per-row Vec headers coexist with
    // shared native columns. The metadata payloads above are separate owners.
    q.wire = sum(
        q.wire,
        sum(
            vector::<EntityExtraImage>(partition.entity_arena.extra.len())?,
            vector::<RelationExtraImage>(partition.relation_arena.extra.len())?,
        )?,
    )?;
    q.wire = sum(
        q.wire,
        sum(
            vector::<Vec<VersionedEntityMetadataImage>>(partition.entity_arena.extra.len())?,
            vector::<Vec<VersionedRelationMetadataImage>>(partition.relation_arena.extra.len())?,
        )?,
    )?;
    q.wire = sum(q.wire, adjacency(partition, request)?)?;
    // Backing capacity is not executed work. Price the actual exported values,
    // while keeping the conservative simultaneous backing quote unchanged.
    super::current_work::charge(partition, contracts, request)?;
    Ok(q)
}

fn state_wire(
    state: Option<&AuthoritativeRecordAspectState>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64> {
    let Some(state) = state else {
        return Ok(0);
    };
    let mut bytes = vector::<PortableRecordAspectStateEntry>(state.aspects().len())?;
    for (_, value) in state.aspects().entries() {
        work(request, 1)?;
        bytes = sum(
            bytes,
            u64::try_from(value.key().as_str().len()).map_err(|_| overflow())?,
        )?;
        match value.view() {
            View::Scalar(value) => {
                bytes = sum(
                    bytes,
                    value
                        .clone_retained_storage_bytes()
                        .and_then(|n| u64::try_from(n).ok())
                        .ok_or_else(overflow)?,
                )?;
            }
            View::Struct(value) => {
                // Export shares the immutable tree. The independently retained
                // portable image keeps its reachable backing until readmission.
                bytes = sum(
                    bytes,
                    u64::try_from(value.owned_allocation_capacity_bytes())
                        .map_err(|_| overflow())?,
                )?;
            }
        }
    }
    Ok(bytes)
}
fn diagnostics(partition: &PartitionState, request: &ExecutionRequest<'_, '_>) -> Result<u64> {
    let mut bytes = 0;
    for column in [
        &partition.entity_arena.diagnostics_enrichment,
        &partition.relation_arena.diagnostics_enrichment,
    ] {
        bytes = sum(bytes, vector::<BTreeMap<Symbol, String>>(column.len())?)?;
        for row in column.iter() {
            work(request, 1)?;
            bytes = sum(bytes, tree::<Symbol, String>(row.len())?)?;
            for value in row.values() {
                work(request, 1)?;
                bytes = sum(bytes, value.capacity() as u64)?;
            }
        }
    }
    Ok(bytes)
}
fn adjacency(partition: &PartitionState, request: &ExecutionRequest<'_, '_>) -> Result<u64> {
    use crate::identity::data::{KindId, RelationId, VersionId};
    let mut bytes = 0;
    for table in [&partition.adjacency, &partition.reverse_adjacency] {
        let mut count = 0usize;
        for (_, set) in table.iter() {
            work(request, 1)?;
            count = count.checked_add(1).ok_or_else(overflow)?;
            bytes = sum(bytes, vector::<RelationId>(set.current_ids().len())?)?;
            bytes = sum(
                bytes,
                vector::<(KindId, VersionId)>(set.structural_revision_count())?,
            )?;
        }
        bytes = sum(
            bytes,
            vector::<crate::durability::data::DurableAdjacencyEntry>(count)?,
        )?;
        bytes = sum(bytes, vector::<(usize, AdjacencySet)>(count)?)?;
    }
    for bits in [
        &partition.entity_arena.live_bitset,
        &partition.entity_arena.reclaimable_bitset,
        &partition.relation_arena.live_bitset,
        &partition.relation_arena.reclaimable_bitset,
    ] {
        let mut count = 0usize;
        for _ in bits.words() {
            work(request, 1)?;
            count = count.checked_add(1).ok_or_else(overflow)?;
        }
        bytes = sum(bytes, vector::<(u64, u64)>(count)?)?;
    }
    Ok(bytes)
}
