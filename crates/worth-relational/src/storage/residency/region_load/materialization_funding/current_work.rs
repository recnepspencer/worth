//! Work of exporting actual current values, independent of spare backing.
use super::*;
use crate::identity::data::{KindId, RelationId, VersionId};
use crate::storage::data::{RecordLifecycleState, RelationalFieldRevision};
use crate::symbols::data::Symbol;
use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
    AuthoritativeRecordAspectState, ContractValidatedAspectValueView as View, FieldKey,
    PortableRecordAspectStateEntry,
};

type Contracts<'a> =
    BTreeMap<(&'a AspectKey, AspectIdentity, AspectContractRevision), &'a AspectContract>;

pub(super) fn charge(
    partition: &PartitionState,
    contracts: &[AspectContract],
    request: &ExecutionRequest<'_, '_>,
) -> Result<()> {
    // This borrowed index is pricing scratch only. Foundation still selects
    // and validates the actual catalog contract during the subsequent export.
    let _index_memory = Funding::reserve(
        tree::<(&AspectKey, AspectIdentity, AspectContractRevision), &AspectContract>(
            contracts.len(),
        )?,
        request,
    )?;
    let mut longest_key = 0;
    for contract in contracts {
        work(request, 1)?;
        longest_key = longest_key.max(contract.key().as_str().len() as u64);
    }
    let navigation = navigation(contracts.len(), sum(1, times(longest_key, 2)?)?)?;
    work(request, times(navigation, contracts.len())?)?;
    work(
        request,
        array::<(
            &AspectKey,
            AspectIdentity,
            AspectContractRevision,
            &AspectContract,
        )>(contracts.len())?,
    )?;
    let mut index: Contracts<'_> = BTreeMap::new();
    for contract in contracts {
        work(request, 1)?;
        index.insert(
            (contract.key(), contract.identity(), contract.revision()),
            contract,
        );
    }
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
        state_work(state, &index, longest_key, request)?;
    }
    for count in [e.generations.len(), r.generations.len()] {
        work(
            request,
            array::<(
                u64,
                u32,
                RecordLifecycleState,
                Option<KindId>,
                VersionId,
                Option<VersionId>,
                [u32; 3],
            )>(count)?,
        )?;
    }
    work(request, array::<EntityExtraImage>(e.extra.len())?)?;
    work(request, array::<RelationExtraImage>(r.extra.len())?)?;
    for rows in e.metadata_history.iter() {
        work(
            request,
            sum(
                size_of::<Vec<VersionedEntityMetadataImage>>() as u64,
                array::<VersionedEntityMetadataImage>(rows.len())?,
            )?,
        )?;
    }
    for rows in r.metadata_history.iter() {
        work(
            request,
            sum(
                size_of::<Vec<VersionedRelationMetadataImage>>() as u64,
                array::<VersionedRelationMetadataImage>(rows.len())?,
            )?,
        )?;
    }
    for rows in [&e.aspect_versions, &r.aspect_versions] {
        for row in rows.iter() {
            work(
                request,
                sum(
                    size_of::<Vec<(Symbol, u64)>>() as u64,
                    array::<(Symbol, u64)>(row.len())?,
                )?,
            )?;
        }
    }
    for rows in [&e.field_revisions, &r.field_revisions] {
        for row in rows.iter() {
            let count = row.as_ref().map_or(0, |row| row.len());
            work(
                request,
                sum(
                    size_of::<Option<Vec<((Symbol, Symbol), RelationalFieldRevision)>>>() as u64,
                    array::<((Symbol, Symbol), RelationalFieldRevision)>(count)?,
                )?,
            )?;
        }
    }
    for column in [&e.diagnostics_enrichment, &r.diagnostics_enrichment] {
        for row in column.iter() {
            work(request, array::<(Symbol, String)>(row.len())?)?;
            for value in row.values() {
                work(request, sum(1, value.len() as u64)?)?;
            }
        }
    }
    for table in [&partition.adjacency, &partition.reverse_adjacency] {
        for (_, set) in table.iter() {
            work(
                request,
                size_of::<crate::durability::data::DurableAdjacencyEntry>() as u64,
            )?;
            work(request, array::<RelationId>(set.current_ids().len())?)?;
            work(
                request,
                array::<(KindId, VersionId)>(set.structural_revision_count())?,
            )?;
        }
    }
    for bits in [
        &e.live_bitset,
        &e.reclaimable_bitset,
        &r.live_bitset,
        &r.reclaimable_bitset,
    ] {
        for _ in bits.words() {
            work(request, size_of::<(u64, u64)>() as u64)?;
        }
    }
    Ok(())
}

fn navigation(count: usize, comparison: u64) -> Result<u64> {
    let levels = count.checked_add(1).ok_or_else(overflow)?.ilog2() as usize + 1;
    times(comparison, levels.checked_mul(22).ok_or_else(overflow)?)
}

fn state_work(
    state: Option<&AuthoritativeRecordAspectState>,
    contracts: &Contracts<'_>,
    longest_key: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<()> {
    let Some(state) = state else { return Ok(()) };
    // Actual entries and their possible Vec relocation, not reserved capacity.
    work(
        request,
        times(
            array::<PortableRecordAspectStateEntry>(state.aspects().len())?,
            2,
        )?,
    )?;
    for (_, value) in state.aspects().entries() {
        let key = value.key().as_str().len() as u64;
        work(
            request,
            navigation(contracts.len(), sum(1, sum(key, longest_key)?)?)?,
        )?;
        // Exact-query clone, portable-basis key, and a possible denial key.
        work(request, times(key, 3)?)?;
        let selected = contracts
            .get(&(
                value.key(),
                value.contract_identity(),
                value.contract_revision(),
            ))
            .copied();
        let selected = match selected {
            Some(contract) => Some(contract),
            None => {
                let mut found = None;
                for ((candidate, _, _), contract) in contracts.iter().rev() {
                    // Both this borrowed preflight and Foundation's fallback.
                    work(
                        request,
                        times(sum(1, sum(candidate.as_str().len() as u64, key)?)?, 2)?,
                    )?;
                    if *candidate == value.key() {
                        found = Some(*contract);
                        break;
                    }
                }
                found
            }
        };
        let Some(contract) = selected else { continue };
        let nested = crate::durability::checkpoints::aspect_state_images::contract_clone_bytes(
            contract, request,
        )
        .map_err(Failure::Request)?;
        work(request, sum(size_of::<AspectContract>() as u64, nested)?)?;
        match value.view() {
            View::Scalar(value) => work(
                request,
                sum(
                    size_of::<AspectValue>() as u64,
                    value.semantic_byte_width() as u64,
                )?,
            )?,
            View::Struct(value) => {
                work(request, array::<(FieldKey, AspectValue)>(value.len())?)?;
                for (key, scalar) in value.fields() {
                    work(
                        request,
                        sum(
                            1,
                            sum(
                                key.as_str().len() as u64,
                                scalar.semantic_byte_width() as u64,
                            )?,
                        )?,
                    )?;
                }
            }
        }
    }
    Ok(())
}
