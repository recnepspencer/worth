use super::*;

pub(super) fn copy_source(
    root: &Path,
    current: &OfflineRootManifestFacts,
    intent: &CopyIntentFact,
    walk: &mut BoundedMediaWalk,
    observations: &mut Vec<OfflineArtifactObservation>,
) -> Result<ChildExpectation, Outcome> {
    if current.generation < intent.source_root {
        return Err(mismatch());
    }
    let source = graph(
        root,
        intent.source_root,
        current.format,
        None,
        walk,
        observations,
    )?;
    source
        .routes
        .into_iter()
        .find(|route| {
            copy_route(
                route,
                intent.source,
                intent.extent,
                intent.source_generation,
                intent.record,
                intent.payload_bytes,
            )
        })
        .ok_or_else(mismatch)
}

pub(super) fn prove_copy_destination(
    root: &Path,
    current: &OfflineRootManifestFacts,
    final_copy: &CopyFinalFact,
    walk: &mut BoundedMediaWalk,
    observations: &mut Vec<OfflineArtifactObservation>,
) -> Result<(), Outcome> {
    let destination = graph(
        root,
        final_copy.result_root,
        current.format,
        None,
        walk,
        observations,
    )?;
    let intent = final_copy.intent;
    destination
        .routes
        .iter()
        .any(|route| {
            copy_route(
                route,
                intent.destination,
                intent.extent,
                intent.destination_generation,
                intent.record,
                intent.payload_bytes,
            )
        })
        .then_some(())
        .ok_or_else(mismatch)
}

pub(super) fn copy_route(
    route: &ChildExpectation,
    range: (u64, u64, u64),
    extent: u64,
    generation: u64,
    record: [u8; 24],
    logical_bytes: u64,
) -> bool {
    route.path == arena_path(range.0)
        && route.offset == range.1
        && route.generation == generation
        && matches!(route.scope,ChildScope::ExtentManifest {arena,extent:found,allocated_bytes,record:found_record,
            logical_bytes:found_bytes} if arena==range.0 && found==extent && allocated_bytes==range.2
            && found_record==record && found_bytes==logical_bytes)
}

pub(super) fn observe_copy_destination_staging(
    root: &Path,
    intent: &CopyIntentFact,
    final_copy: Option<CopyFinalFact>,
    walk: &mut BoundedMediaWalk,
    observations: &mut Vec<OfflineArtifactObservation>,
) {
    if final_copy.is_some() {
        return;
    }
    let path = arena_path(intent.destination.0);
    if !matches!(root.join(&path).try_exists(), Ok(true)) {
        return;
    }
    let outcome = Outcome::Unknown(
        crate::integrity_observation::OfflineUnknownPhysicalReason::ParentScopeUnavailable,
    );
    walk.record_outcome(&outcome);
    observations.push(OfflineArtifactObservation::new(
        path.clone(),
        Family::ExtentArenaFrame.into(),
        PhysicalArtifactIdentity::new(format!("copy-staging:{}:{path}", intent.lsn)).unwrap(),
        PhysicalArtifactGeneration::encoded(intent.destination_generation).unwrap(),
        PhysicalByteRange::new(intent.destination.1, intent.destination.2).ok(),
        outcome,
    ));
}

pub(super) fn copy_observation(
    intent: CopyIntentFact,
    outcome: Outcome,
) -> OfflineArtifactObservation {
    let path = arena_path(intent.source.0);
    OfflineArtifactObservation::new(
        path.clone(),
        Family::ExtentArenaFrame.into(),
        PhysicalArtifactIdentity::new(format!("copy-intent:{}:{path}", intent.lsn)).unwrap(),
        PhysicalArtifactGeneration::encoded(intent.source_generation).unwrap(),
        PhysicalByteRange::new(intent.source.1, intent.source.2).ok(),
        outcome,
    )
}

pub(super) fn admit_rewrite(
    root: &Path,
    current: &OfflineRootManifestFacts,
    rewrite: &super::super::rewrite::RewriteFact,
    walk: &mut BoundedMediaWalk,
    observations: &mut Vec<OfflineArtifactObservation>,
) -> Result<ChildExpectation, Outcome> {
    let source = graph(
        root,
        rewrite.source_root,
        current.format,
        None,
        walk,
        observations,
    )?;
    let destination = graph(
        root,
        rewrite.result_root,
        current.format,
        None,
        walk,
        observations,
    )?;
    let matches = |route: &ChildExpectation, generation: u64, range: (u64, u64, u64)| {
        route.path == arena_path(range.0)
            && route.offset == range.1
            && route.generation == generation
            && matches!(route.scope,ChildScope::ExtentManifest { arena,extent,allocated_bytes,record,logical_bytes }
                if arena == range.0 && extent == rewrite.extent && allocated_bytes == range.2 && record == rewrite.record
                    && logical_bytes == rewrite.logical_bytes)
    };
    let held = source
        .routes
        .into_iter()
        .find(|route| matches(route, rewrite.source_generation, rewrite.source))
        .ok_or_else(mismatch)?;
    if !destination
        .routes
        .iter()
        .any(|route| matches(route, rewrite.destination_generation, rewrite.destination))
    {
        return Err(mismatch());
    }
    Ok(held)
}

pub(super) fn admit_intent(
    root: &Path,
    current: &OfflineRootManifestFacts,
    intent: &RetirementFact,
    walk: &mut BoundedMediaWalk,
    observations: &mut Vec<OfflineArtifactObservation>,
) -> Result<Option<ChildExpectation>, Outcome> {
    if current.generation < intent.releasing_root {
        return Err(mismatch());
    }
    let source = graph(
        root,
        intent.source_root,
        current.format,
        None,
        walk,
        observations,
    )?;
    let held = if let Some((arena, offset, length)) = intent.range {
        let path = format!("families/records/arenas/arena-{arena:016x}.data");
        Some(source.routes.into_iter().find(|route| route.path == path && route.offset == offset
            && route.generation == intent.generation
            && matches!(route.scope, ChildScope::ExtentManifest { extent, allocated_bytes, .. }
                if extent == intent.id && allocated_bytes == length)).ok_or_else(mismatch)?)
    } else {
        if source.capacity == 0
            || source.capacity != intent.bytes
            || source
                .routes
                .iter()
                .any(|route| route.path == arena_path(intent.id))
            || !source.free.iter().any(|&(arena, offset, length, _)| {
                arena == intent.id && offset == 0 && length == source.capacity
            })
        {
            return Err(mismatch());
        }
        None
    };
    if current.generation < intent.candidate {
        return Ok(held);
    }
    let released = graph(
        root,
        intent.candidate,
        current.format,
        Some(intent.digest),
        walk,
        observations,
    )?;
    if let Some((arena, offset, length)) = intent.range {
        let end = offset.checked_add(length).ok_or_else(mismatch)?;
        if !released.free.iter().any(|&(id, start, bytes, generation)| {
            id == arena
                && start <= offset
                && start
                    .checked_add(bytes)
                    .is_some_and(|free_end| free_end >= end)
                && generation > intent.source_root
                && generation <= intent.candidate
        }) {
            return Err(mismatch());
        }
    } else if released
        .free
        .iter()
        .any(|&(arena, _, _, _)| arena == intent.id)
        || released
            .routes
            .iter()
            .any(|route| route.path == arena_path(intent.id))
    {
        return Err(mismatch());
    }
    Ok(None)
}

pub(super) fn graph(
    root: &Path,
    generation: u64,
    format: [u8; 10],
    digest: Option<[u8; 32]>,
    walk: &mut BoundedMediaWalk,
    observations: &mut Vec<OfflineArtifactObservation>,
) -> Result<Graph, Outcome> {
    let path = format!("families/records/roots/root-{generation:016x}.manifest");
    let acquired = walk.acquire_range(&root.join(&path), 4, 0, 384)?;
    if acquired.is_alias() || acquired.byte_length != 384 {
        return Err(mismatch());
    }
    if let Some(digest) = digest {
        walk.counters_mut().checksum_calculations += 1;
        if crate::integrity_observation::sha256::sha256(&acquired.bytes) != digest {
            return Err(mismatch());
        }
    }
    let manifest = read_root_manifest(&acquired.bytes, walk.counters_mut())?;
    if manifest.generation != generation || manifest.format != format {
        return Err(mismatch());
    }
    let mut queue = VecDeque::from(root_children(&manifest));
    let mut visited = BTreeSet::new();
    let mut graph = Graph::default();
    while let Some(expected) = queue.pop_front() {
        if !matches!(
            expected.family,
            Family::RootRoutingBlock | Family::FreeSpaceHeader | Family::FreeSpaceMembershipBlock
        ) {
            continue;
        }
        if !visited.insert((expected.path.clone(), expected.offset)) {
            continue;
        }
        if visited.len() as u64 > walk.maximum_entries() {
            return Err(walk.entry_bound());
        }
        let acquired = walk.acquire(&root.join(&expected.path), 4)?;
        if acquired.is_alias() {
            return Err(mismatch());
        }
        let children = inspect_expected(&acquired.bytes, &expected, walk)?;
        match expected.scope {
            ChildScope::FreeSpace { .. } => graph.capacity = read_u64(&acquired.bytes, 200),
            ChildScope::Tree { level: 0, .. }
                if expected.family == Family::FreeSpaceMembershipBlock =>
            {
                let count = usize::from(read_u16(&acquired.bytes, 66));
                for entry in acquired.bytes[88..88 + count * 40].chunks_exact(40) {
                    if entry[0] == 2 {
                        graph.free.push((
                            read_u64(entry, 8),
                            read_u64(entry, 16),
                            read_u64(entry, 24),
                            read_u64(entry, 32),
                        ));
                    }
                }
            }
            _ => {}
        }
        for child in children {
            if child.family == Family::ExtentManifest {
                graph.routes.push(child);
            } else {
                queue.push_back(child);
            }
        }
        if queue
            .len()
            .saturating_add(graph.routes.len())
            .saturating_add(graph.free.len()) as u64
            > walk.maximum_entries()
        {
            return Err(walk.entry_bound());
        }
        observations.push(project(&expected, acquired.bytes.len(), Outcome::Intact));
    }
    Ok(graph)
}

pub(super) fn arena_path(arena: u64) -> String {
    format!("families/records/arenas/arena-{arena:016x}.data")
}
pub(super) fn mismatch() -> Outcome {
    damage(Cause::ScopeMismatch, None, Blast::Artifact)
}
pub(super) fn evidence_observation(
    path: String,
    generation: u64,
    outcome: Outcome,
) -> OfflineArtifactObservation {
    OfflineArtifactObservation::new(
        path.clone(),
        Family::ExtentArenaFrame.into(),
        PhysicalArtifactIdentity::new(format!("retirement:{generation}:{path}")).unwrap(),
        PhysicalArtifactGeneration::encoded(generation).unwrap(),
        PhysicalByteRange::new(0, 0).ok(),
        outcome,
    )
}
