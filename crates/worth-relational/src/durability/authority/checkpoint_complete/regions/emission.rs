//! Each actual Cell/schema pair emits once; all ACK sources stay strongly held.
use super::*;
use worth_execution::ExecutionRequestArrayBuilder;
#[derive(Clone)]
struct Cached {
    leaf: RelationalCheckpointRegion,
    publication: Option<RelationalCheckpointRegionPublication>,
}
pub(super) fn capture(
    grouped: Grouped<'_>,
    roots: &CapturedRoots,
    base: Option<&crate::durability::checkpoint_admission::FrozenSelectedCaptureBase>,
    mirror_source: CheckpointMirror<'_>,
    dictionary: &dyn SymbolTextLookup,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RegionOutput, Denial> {
    let covered = super::covered::CoveredRegions::capture(base, request)?;
    let mut output =
        PreparedCheckpointRoots::prepare(grouped.groups.len(), request).map_err(request_stop)?;
    let (empty, empty_memory) = grouped.copy_empty(request)?;
    let mirror_count = match &mirror_source {
        CheckpointMirror::CurrentMainRoot => grouped
            .mirror
            .and_then(|commit| roots.get(commit))
            .map_or(0, |root| root.checkpoint_region_count()),
        CheckpointMirror::Substrate { edition, .. } => edition.cells().count(),
    };
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
            source.checkpoint_region_count(),
            request,
        )
        .map_err(request_stop)?;
        source.try_visit_checkpoint_regions(|region| {
            crate::execution::check_request(request).map_err(request_stop)?;
            region.cell.check_damage().map_err(|failure| {
                crate::durability::checkpoint_history::capture::capture_stop(failure.into())
            })?;
            #[expect(clippy::disallowed_methods, reason = "Fixed allocation identity deduplicates custody; canonical order never uses this address")]
            let identity = Arc::as_ptr(&region.cell) as usize;
            let key = (identity, group.carrier);
            if cache.get(&key).is_none() {
                let captured = match covered.get(*commit, region.partition_id, group.carrier) {
                    Some(leaf) => {
                        for locator in [leaf.head, leaf.history] {
                            if members.get(&locator).is_none() {
                                members.insert_cloned_native_for_request(
                                    &locator,
                                    &RelationalCheckpointMember::Carried { locator },
                                    *request,
                                ).map_err(map_stop)?;
                            }
                        }
                        Cached { leaf: leaf.clone(), publication: None }
                    }
                    None => {
                        let captured = region.cell.capture_complete_for_request(
                            request, &group.schema, group.carrier, dictionary,
                        ).map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
                        for member in captured.members {
                            let locator = member.locator();
                            if members.get(&locator).is_none() {
                                members.insert_cloned_native_for_request(&locator, &member, *request)
                                    .map_err(map_stop)?;
                            }
                        }
                        Cached { leaf: captured.leaf, publication: Some(captured.publication) }
                    }
                };
                cache
                    .insert_cloned_native_for_request(
                        &key,
                        &captured,
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
            if group.current_main && matches!(mirror_source, CheckpointMirror::CurrentMainRoot) {
                mirror.push(cached.leaf.clone());
            }
            Ok::<_, Denial>(())
        })?;
        if group.current_main && matches!(mirror_source, CheckpointMirror::CurrentMainRoot) {
            mirror_schema = Some(group.carrier);
        }
        output
            .push(
                root.finish(
                    group.carrier,
                    crate::durability::checkpoints::content::root_capture::capture(
                        request,
                        source.retained_content(),
                        members,
                    )?,
                )
                .map_err(request_stop)?,
            )
            .map_err(request_stop)?;
    }
    if let CheckpointMirror::Substrate {
        edition,
        schema,
        carrier,
    } = mirror_source
    {
        for (partition, retention) in edition.cells() {
            crate::execution::check_request(request).map_err(request_stop)?;
            let cell = retention.cell();
            #[expect(
                clippy::disallowed_methods,
                reason = "Fixed allocation identity deduplicates custody; canonical order never uses this address"
            )]
            let identity = Arc::as_ptr(cell) as usize;
            let key = (identity, carrier);
            if !cache.contains_key(&key) {
                let lease = retention.demand_full(request, 1 << 30).map_err(|cause| {
                    crate::durability::checkpoint_history::capture::capture_stop(cause.into())
                })?;
                let captured = lease
                    .capture_checkpoint_data_for_request(request, schema, carrier, dictionary)
                    .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
                for member in captured.members {
                    let locator = member.locator();
                    if !members.contains_key(&locator) {
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
                            publication: Some(captured.publication),
                        },
                        *request,
                    )
                    .map_err(map_stop)?;
                if !cells.contains_key(&identity) {
                    cells
                        .insert_cloned_native_for_request(&identity, cell, *request)
                        .map_err(map_stop)?;
                }
            }
            let leaf = &cache
                .get(&key)
                .ok_or(Denial::OriginalAuthorityUnavailable)?
                .leaf;
            if leaf.partition_id != partition {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            mirror.push(leaf.clone());
        }
        mirror_schema = Some(carrier);
    }
    if mirror.len() != mirror_count {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    let publication_count = cache
        .values()
        .filter(|entry| entry.publication.is_some())
        .count();
    let mut publications =
        ExecutionRequestArrayBuilder::allocate(*request, publication_count).map_err(array_stop)?;
    for (_, entry) in cache.iter() {
        if let Some(publication) = &entry.publication {
            publications.push(publication.clone()).map_err(array_stop)?;
        }
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
