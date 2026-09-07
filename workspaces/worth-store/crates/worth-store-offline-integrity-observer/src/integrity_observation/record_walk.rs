use super::child_expectation::{ChildExpectation, ChildScope};
use super::families::{
    bootstrap_catalog::read_bootstrap_catalog,
    durable_frame::{read_u16, read_u32},
    extent::{read_extent_chunk, read_extent_manifest},
    free_space::{read_free_space_header, read_free_space_membership},
    page_frame::read_page_frame,
    root_manifest::OfflineRootManifestFacts,
    root_routing::read_root_routing,
    segment_membership::read_segment_membership,
    tree_reference::reference,
};
use super::{
    BoundedMediaWalk, OfflineArtifactDuplicateEvidence, OfflineArtifactObservation,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflinePhysicalDamageLocalization,
    OfflineUnknownPhysicalReason,
};
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use worth_foundational::{
    PhysicalArtifactFamily as Family, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
    PhysicalByteRange,
};

pub(crate) fn observe_records(
    root: &Path,
    store: Option<[u8; 16]>,
    roots: &[OfflineRootManifestFacts],
    walk: &mut BoundedMediaWalk,
) -> Vec<OfflineArtifactObservation> {
    let mut observations = vec![observe_bootstrap(root, store, walk)];
    if walk.exhausted_reason().is_some() {
        return observations;
    }
    let mut queue = VecDeque::new();
    for manifest in roots {
        queue.extend(root_children(manifest));
    }
    let mut visited: BTreeMap<(String, u64), ChildExpectation> = BTreeMap::new();
    while let Some(expected) = queue.pop_front() {
        let key = (expected.path.clone(), expected.offset);
        if let Some(prior) = visited.get(&key) {
            if prior != &expected {
                observations.push(project(
                    &expected,
                    0,
                    damage(Cause::ScopeMismatch, None, Blast::Artifact),
                ));
            }
            continue;
        }
        if visited.len() as u64 >= walk.maximum_entries() {
            observations.push(project(&expected, 0, walk.entry_bound()));
            break;
        }
        visited.insert(key, expected.clone());
        let path = root.join(&expected.path);
        let acquired = if !path.try_exists().unwrap_or(true) {
            walk.counters_mut().missing_artifacts += 1;
            Err(damage(Cause::MissingArtifact, None, Blast::Artifact))
        } else {
            walk.acquire(&path, 4)
        };
        let mut alias = None;
        let mut length = 0;
        let outcome = match acquired {
            Err(outcome) => outcome,
            Ok(acquired) => {
                alias = acquired
                    .physical_alias_of
                    .as_ref()
                    .map(|path| super::unknown_artifact::relative_path(root, path));
                if let ChildScope::Page { pages, .. } = expected.scope {
                    let expected_bytes =
                        u64::from(pages) * u64::from(read_u32(&expected.format, 2));
                    if acquired.byte_length as u64 != expected_bytes {
                        let outcome = damage(
                            Cause::Framing,
                            Some((0, acquired.byte_length.max(1) as u64)),
                            Blast::Artifact,
                        );
                        walk.record_outcome(&outcome);
                        observations.push(project(&expected, 0, outcome));
                        continue;
                    }
                }
                let end = expected
                    .length
                    .and_then(|length| expected.offset.checked_add(length))
                    .unwrap_or(acquired.byte_length as u64);
                let range = usize::try_from(expected.offset)
                    .ok()
                    .zip(usize::try_from(end).ok())
                    .and_then(|(start, end)| acquired.bytes.get(start..end));
                match range {
                    None => damage(
                        Cause::Truncation,
                        Some((
                            acquired.byte_length as u64,
                            end.saturating_sub(acquired.byte_length as u64).max(1),
                        )),
                        Blast::Artifact,
                    ),
                    Some(bytes) => {
                        length = bytes.len();
                        let result = inspect_expected(bytes, &expected, walk);
                        match result {
                            Err(outcome) => shift_outcome(outcome, expected.offset),
                            Ok(children) => {
                                if queue.len().saturating_add(children.len()) as u64
                                    > walk.maximum_entries()
                                {
                                    walk.entry_bound()
                                } else {
                                    queue.extend(children);
                                    Outcome::Intact
                                }
                            }
                        }
                    }
                }
            }
        };
        walk.record_outcome(&outcome);
        let mut observation = project(&expected, length, outcome);
        if let Some(first_path) = alias {
            observation =
                observation.with_duplicate(OfflineArtifactDuplicateEvidence::PhysicalAlias {
                    first_path: first_path.into(),
                });
        }
        observations.push(observation);
    }
    observations
}

fn root_children(root: &OfflineRootManifestFacts) -> Vec<ChildExpectation> {
    use super::families::durable_frame::read_u64;
    let payload = &root.payload;
    let tree = read_u64(payload, 8);
    let capacity = read_u16(payload, 16);
    let mut children = Vec::new();
    if payload[40] == 1 {
        children.push(reference(
            &payload[48..120],
            Family::RootRoutingBlock,
            tree,
            capacity,
            root.format,
        ));
    }
    if payload[160] == 1 {
        children.push(reference(
            &payload[168..224],
            Family::SegmentMembershipBlock,
            tree,
            capacity,
            root.format,
        ));
    }
    children.push(ChildExpectation {
        path: format!(
            "families/records/free-space/free-space-{:016x}.manifest",
            root.generation
        ),
        family: Family::FreeSpaceHeader,
        generation: root.generation,
        format: root.format,
        offset: 0,
        length: Some(176),
        checksum: Some(read_u32(payload, 152)),
        scope: ChildScope::FreeSpace { tree, capacity },
    });
    // Root's free-space reference is independently bound as well as the header reference.
    if payload[232] == 1 {
        children.push(reference(
            &payload[240..296],
            Family::FreeSpaceMembershipBlock,
            tree,
            capacity,
            root.format,
        ));
    }
    children
}

fn inspect_expected(
    bytes: &[u8],
    expected: &ChildExpectation,
    walk: &mut BoundedMediaWalk,
) -> Result<Vec<ChildExpectation>, Outcome> {
    let maximum_children = walk.maximum_entries();
    let counters = walk.counters_mut();
    let children = match expected.family {
        Family::RootRoutingBlock => read_root_routing(bytes, expected, counters),
        Family::SegmentMembershipBlock => read_segment_membership(bytes, expected, counters),
        Family::FreeSpaceHeader => read_free_space_header(bytes, expected, counters),
        Family::FreeSpaceMembershipBlock => read_free_space_membership(bytes, expected, counters),
        Family::PageFrame => read_page_frame(bytes, expected, counters),
        Family::ExtentManifest => read_extent_manifest(bytes, expected, maximum_children, counters),
        Family::ExtentChunkFrame => read_extent_chunk(bytes, expected, counters),
        _ => unreachable!("record children have closed family dispatch"),
    }?;
    if let Some(checksum) = expected.checksum {
        counters.checksum_calculations += 1;
        if super::crc32c::crc32c(&[bytes]) != checksum {
            return Err(damage(
                Cause::ChecksumMismatch,
                Some((0, bytes.len() as u64)),
                Blast::Artifact,
            ));
        }
    }
    Ok(children)
}

fn observe_bootstrap(
    root: &Path,
    store: Option<[u8; 16]>,
    walk: &mut BoundedMediaWalk,
) -> OfflineArtifactObservation {
    let relative = "families/records/bootstrap.catalog";
    let path = root.join(relative);
    let mut length = 0;
    let outcome = if let Some(reason) = walk.exhausted_reason() {
        Outcome::Indeterminate(reason)
    } else if !path.try_exists().unwrap_or(true) {
        walk.counters_mut().missing_artifacts += 1;
        damage(Cause::MissingArtifact, None, Blast::Artifact)
    } else {
        match walk.acquire(&path, 3) {
            Err(outcome) => outcome,
            Ok(acquired) => {
                length = acquired.byte_length;
                match store {
                    None => {
                        Outcome::Unknown(OfflineUnknownPhysicalReason::StoreIdentityUnavailable)
                    }
                    Some(store) => {
                        read_bootstrap_catalog(&acquired.bytes, store, walk.counters_mut())
                            .map_or_else(|outcome| outcome, |()| Outcome::Intact)
                    }
                }
            }
        }
    };
    walk.record_outcome(&outcome);
    OfflineArtifactObservation::new(
        relative,
        Family::BootstrapCatalog.into(),
        PhysicalArtifactIdentity::new("bootstrap-catalog").unwrap(),
        PhysicalArtifactGeneration::NotEncoded,
        PhysicalByteRange::new(0, length as u64).ok(),
        outcome,
    )
}

fn project(
    expected: &ChildExpectation,
    length: usize,
    outcome: Outcome,
) -> OfflineArtifactObservation {
    OfflineArtifactObservation::new(
        expected.path.clone(),
        expected.family.into(),
        PhysicalArtifactIdentity::new(expected.identity()).unwrap(),
        PhysicalArtifactGeneration::encoded(expected.generation)
            .unwrap_or(PhysicalArtifactGeneration::NotEncoded),
        PhysicalByteRange::new(expected.offset, expected.length.unwrap_or(length as u64)).ok(),
        outcome,
    )
}

pub(crate) fn damage(cause: Cause, range: Option<(u64, u64)>, blast: Blast) -> Outcome {
    Outcome::Damaged(OfflinePhysicalDamageLocalization::new(
        cause, range, None, blast,
    ))
}

pub(crate) fn shift_outcome(outcome: Outcome, offset: u64) -> Outcome {
    match outcome {
        Outcome::Damaged(value) => Outcome::Damaged(OfflinePhysicalDamageLocalization::new(
            value.cause(),
            value
                .damaged_range()
                .map(|range| (offset.saturating_add(range.offset()), range.length())),
            value.field(),
            value.blast_radius(),
        )),
        Outcome::Unsupported(value) => {
            Outcome::Unsupported(super::OfflineUnsupportedPhysicalVersion::new(
                value.axis(),
                value.observed(),
                value.supported(),
                PhysicalByteRange::new(
                    offset.saturating_add(value.range().offset()),
                    value.range().length(),
                )
                .unwrap(),
            ))
        }
        other => other,
    }
}
