//! Each actual Cell/schema pair emits once; all ACK sources stay strongly held.
use super::*;
use worth_execution::ExecutionRequestArrayBuilder;
#[derive(Clone)]
struct Cached {
    leaf: RelationalCheckpointRegion,
    publication: RelationalCheckpointRegionPublication,
}
pub(super) fn capture(
    grouped: Grouped<'_>,
    roots: &FoldedRoots,
    dictionary: &dyn SymbolTextLookup,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RegionOutput, Denial> {
    let mut output =
        PreparedCheckpointRoots::prepare(grouped.groups.len(), request).map_err(request_stop)?;
    let (empty, empty_memory) = grouped.copy_empty(request)?;
    let mirror_count = grouped
        .mirror
        .and_then(|commit| roots.get(commit))
        .map_or(0, |root| root.root.checkpoint_region_count());
    let mirror_memory = reserve(
        array_bytes::<RelationalCheckpointRegion>(mirror_count)?,
        request,
    )?;
    let mut mirror = Vec::with_capacity(mirror_count);
    let mut mirror_schema = None;
    let mut cache = ExecutionOrderedMap::<(usize, [u8; 32]), Cached>::default();
    let mut cells = ExecutionOrderedMap::<usize, Arc<RelationalRegionResidency>>::default();
    for (commit, group) in grouped.groups.iter() {
        crate::execution::check_request(request).map_err(request_stop)?;
        let source = roots
            .get(*commit)
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let first = grouped
            .descriptors
            .rank_of(&(*commit, group.first_ordinal))
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let descriptors = || {
            (0..group.count).map(|offset| {
            let (key,descriptor)=grouped.descriptors.entry_at(first+offset)
                .expect(/* Proof: grouping admitted each descriptor and its exact per-commit count in this unchanged ordered index. */ "grouped component exists");
            assert_eq!(key.0,*commit,"grouped component has original commit");
            *descriptor
        })
        };
        let mut root = PreparedCheckpointRoot::prepare_from(
            *commit,
            group.count,
            descriptors,
            source.root.checkpoint_region_count(),
            request,
        )
        .map_err(request_stop)?;
        source.root.try_visit_checkpoint_regions(|region| {
            crate::execution::check_request(request).map_err(request_stop)?;
            let identity = Arc::as_ptr(&region.cell) as usize;
            let key = (identity, group.carrier);
            if cache.get(&key).is_none() {
                let captured = region
                    .cell
                    .capture_complete_for_request(request, &group.schema, group.carrier, dictionary)
                    .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
                for member in captured.members {
                    let locator = member.locator();
                    if members.get(&locator).is_none() {
                        members
                            .insert_cloned_native_for_request(&locator, &member, *request)
                            .map_err(map_stop)?;
                    }
                }
                cache
                    .insert_cloned_native_for_request(
                        &key,
                        &Cached {
                            leaf: captured.leaf,
                            publication: captured.publication,
                        },
                        *request,
                    )
                    .map_err(map_stop)?;
                if cells.get(&identity).is_none() {
                    cells
                        .insert_cloned_native_for_request(&identity, &region.cell, *request)
                        .map_err(map_stop)?;
                }
            }
            let cached = cache
                .get(&key)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            if cached.leaf.partition_id != region.partition_id
                || cached.leaf.content_digest != region.content_digest
                || cached.leaf.kind_authority != region.kind_commitment
                || cached.leaf.entity_slots != region.entity_slots
                || cached.leaf.relation_slots != region.relation_slots
                || cached.leaf.unavailable_records != region.unavailable_records
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            root.push(cached.leaf.clone()).map_err(request_stop)?;
            if group.current_main {
                mirror.push(cached.leaf.clone());
            }
            Ok::<_, Denial>(())
        })?;
        if group.current_main {
            mirror_schema = Some(group.carrier);
        }
        output
            .push(root.finish(group.carrier).map_err(request_stop)?)
            .map_err(request_stop)?;
    }
    if mirror.len() != mirror_count {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    let mut publications =
        ExecutionRequestArrayBuilder::allocate(*request, cache.len()).map_err(array_stop)?;
    for (_, entry) in cache.iter() {
        publications
            .push(entry.publication.clone())
            .map_err(array_stop)?;
    }
    let mut roster =
        ExecutionRequestArrayBuilder::allocate(*request, cells.len()).map_err(array_stop)?;
    for (_, cell) in cells.iter() {
        roster.push(cell.clone()).map_err(array_stop)?;
    }
    Ok(RegionOutput {
        roots: output.finish().map_err(request_stop)?,
        empty,
        mirror,
        mirror_schema,
        publications: publications.seal().map_err(array_stop)?,
        cells: roster.seal().map_err(array_stop)?,
        _empty_memory: empty_memory,
        _mirror_memory: mirror_memory,
    })
}
