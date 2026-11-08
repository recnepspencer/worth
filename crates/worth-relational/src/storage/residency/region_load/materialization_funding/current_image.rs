//! Current-state materialization quotes, polled before state/membership visits.
use super::*;
use crate::storage::partition::AdjacencySet;
use crate::symbols::data::Symbol;
use worth_foundational::facade::{
    AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView as View,
    FieldKey, PortableRecordAspectStateEntry,
};
pub(in crate::storage::residency) fn current(
    partition: &PartitionState,
    contract: u64,
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
        q.wire = sum(q.wire, state_wire(state, contract, request)?)?;
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
    contract: u64,
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
            sum(
                value.key().owned_allocation_capacity_bytes() as u64,
                contract,
            )?,
        )?;
        match value.view() {
            View::Scalar(value) => {
                bytes = sum(bytes, value.owned_allocation_capacity_bytes() as u64)?;
            }
            View::Struct(value) => {
                bytes = sum(bytes, vector::<(FieldKey, AspectValue)>(value.len())?)?;
                for (key, scalar) in value.fields() {
                    work(request, 1)?;
                    bytes = sum(
                        bytes,
                        sum(
                            key.owned_allocation_capacity_bytes() as u64,
                            scalar.owned_allocation_capacity_bytes() as u64,
                        )?,
                    )?;
                }
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

/// Local expected-commitment scratch never enters the returned Full region.
/// Each tree item admits two Patricia nodes plus a larger native map node.
/// Hashing streams; only resolved version-name references need a sort vector.
pub(in crate::storage::residency) fn layered_commitment(
    partition: &PartitionState,
    symbols: &crate::symbols::data::StringInterner,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Funding> {
    let cursor_bytes = vector::<[usize; 2]>(16 * usize::BITS as usize)?;
    work(request, cursor_bytes)?;
    let _cursors = Funding::reserve(cursor_bytes, request)?;
    let e = &partition.entity_arena;
    let r = &partition.relation_arena;
    let mut records = 0usize;
    let mut tree_items = 128usize; // One simultaneous copied Patricia spine.
    for (prefix, versions) in e
        .history_prefixes
        .iter()
        .zip(e.metadata_history.iter().map(|row| row.len()))
        .chain(
            r.history_prefixes
                .iter()
                .zip(r.metadata_history.iter().map(|row| row.len())),
        )
    {
        work(request, 1)?;
        records = records.checked_add(1).ok_or_else(overflow)?;
        tree_items = tree_items
            .checked_add(versions)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(overflow)?; // Record root/history directory and all current leaves.
        if let Some(prefix) = prefix {
            tree_items = tree_items
                .checked_add(prefix.history_opening.frontier.siblings.len())
                .ok_or_else(overflow)?;
        }
    }
    let mut scratch = sum(
        cursor_bytes,
        sum(
            vector::<u64>(records)?,
            sum(
                vector::<(u128, [u8; 32])>(records)?,
                vector::<(u64, [usize; 2])>(records)?,
            )?,
        )?,
    )?;
    let mut names_bytes = 0u64;
    let mut version_rows = 0usize;
    for versions in e.aspect_versions.iter().chain(r.aspect_versions.iter()) {
        work(request, 1)?;
        version_rows = version_rows
            .checked_add(versions.len())
            .ok_or_else(overflow)?;
        for (symbol, _) in versions.iter() {
            work(request, usize::BITS as u64)?;
            let name = symbols.resolve(*symbol).ok_or({
                Failure::StorageUnavailable(RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Symbols {
                        first: *symbol,
                        last: *symbol,
                    },
                    cause: RelationalStorageUnavailableCause::SourceUnavailable,
                })
            })?;
            names_bytes = sum(names_bytes, name.len() as u64)?;
        }
    }
    for extra in e.extra.iter() {
        work(request, 1)?;
        if let Some(fingerprint) = extra.structural_fingerprint {
            work(request, usize::BITS as u64)?;
            let name = symbols.resolve(fingerprint.family).ok_or({
                Failure::StorageUnavailable(RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Symbols {
                        first: fingerprint.family,
                        last: fingerprint.family,
                    },
                    cause: RelationalStorageUnavailableCause::SourceUnavailable,
                })
            })?;
            names_bytes = sum(names_bytes, name.len() as u64)?;
        }
    }
    scratch = sum(scratch, vector::<(&str, u64)>(version_rows)?)?;
    for table in [&partition.adjacency, &partition.reverse_adjacency] {
        let mut slots = 0usize;
        for (_, adjacency) in table.iter() {
            work(request, 1)?;
            slots = slots.checked_add(1).ok_or_else(overflow)?;
            let count = adjacency.current_ids().len();
            tree_items = tree_items.checked_add(count).ok_or_else(overflow)?;
            // changed_current_memberships(None) retains its BTreeSet while
            // allocating the output Vec; both coexist with the commitment.
            scratch = sum(
                scratch,
                sum(
                    tree::<crate::identity::data::RelationId, ()>(count)?,
                    vector::<(crate::identity::data::RelationId, bool)>(count)?,
                )?,
            )?;
        }
        tree_items = tree_items.checked_add(slots).ok_or_else(overflow)?;
        scratch = sum(
            scratch,
            sum(
                vector::<u64>(slots)?,
                sum(
                    vector::<(u128, [u8; 32])>(slots)?,
                    vector::<(usize, [usize; 2])>(slots)?,
                )?,
            )?,
        )?;
    }
    for words in [
        &e.live_bitset,
        &e.reclaimable_bitset,
        &r.live_bitset,
        &r.reclaimable_bitset,
    ] {
        let mut count = 0usize;
        for _ in words.words() {
            work(request, 1)?;
            count = count.checked_add(1).ok_or_else(overflow)?;
        }
        tree_items = tree_items.checked_add(count).ok_or_else(overflow)?;
        scratch = sum(
            scratch,
            sum(
                tree::<usize, ()>(count)?,
                vector::<(usize, Option<u64>)>(count)?,
            )?,
        )?;
    }
    // A prefixed history can hold both old and newly assigned last-value
    // carriers during replacement. Quote both, independently of node slack.
    let last_values = times(
        shared::<(u128, [u8; 32], Option<ExecutionMemoryReservation>)>()?,
        records.checked_mul(2).ok_or_else(overflow)?,
    )?;
    let bytes = sum(
        scratch,
        sum(last_values, head_commitment_nodes(tree_items)?)?,
    )?;
    // Current metadata/Extra serialization was already charged by current().
    // Name bytes are independent of Symbol IDs; include lexical sort work.
    work(
        request,
        sum(bytes, times(names_bytes, 2 * usize::BITS as usize)?)?,
    )?;
    Funding::reserve(bytes, request)
}
