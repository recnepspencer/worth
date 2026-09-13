//! Self-contained envelope damage remains observable when no parent establishes child scope.
use super::{
    families::durable_frame::{read_durable_frame, read_u32},
    record_walk::shift_outcome,
    unknown_artifact::relative_path,
    BoundedMediaWalk, OfflineArtifactDuplicateEvidence, OfflineArtifactObservation,
    OfflineIntegrityOutcome as Outcome, OfflineUnknownPhysicalReason as Unknown,
};
use std::path::Path;
use worth_foundational::{
    PhysicalArtifactFamily as Family, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
    PhysicalByteRange,
};
use worth_store_physical_format::integrity_declarations::{
    families as declarations, PhysicalIntegrityFormatDeclaration,
};

struct DeclaredFile {
    family: Family,
    kind: u8,
    generation: u64,
    declaration: PhysicalIntegrityFormatDeclaration,
    fixed_bytes: Option<usize>,
    container: bool,
}

pub(crate) fn observe_unaddressed_record(
    root: &Path,
    path: &Path,
    depth: u32,
    walk: &mut BoundedMediaWalk,
) -> Option<Vec<OfflineArtifactObservation>> {
    let relative = relative_path(root, path);
    let declaration = classify(&relative)?;
    let acquired = match walk.acquire(path, depth) {
        Ok(acquired) => acquired,
        Err(outcome) => {
            walk.record_outcome(&outcome);
            return Some(vec![project(&relative, &declaration, 0, 0, outcome)]);
        }
    };
    if let Some(first) = acquired.physical_alias_of {
        return Some(vec![project(
            &relative,
            &declaration,
            0,
            acquired.byte_length,
            Outcome::Unknown(Unknown::PhysicalAliasNotReinspected),
        )
        .with_duplicate(OfflineArtifactDuplicateEvidence::PhysicalAlias {
            first_path: relative_path(root, &first).into(),
        })]);
    }
    let mut observations = Vec::new();
    let mut offset = 0;
    loop {
        if observations.len() as u64 >= walk.maximum_entries() {
            observations.push(project(
                &relative,
                &declaration,
                offset,
                0,
                walk.entry_bound(),
            ));
            break;
        }
        let remaining = &acquired.bytes[offset..];
        // Unchecked length can narrow only a bounded slice, never allocate or establish scope.
        let length = if declaration.container && remaining.len() >= 48 {
            48_usize.saturating_add(read_u32(remaining, 24) as usize)
        } else {
            declaration.fixed_bytes.unwrap_or(remaining.len().max(48))
        };
        let take = length.min(remaining.len());
        let result = read_durable_frame(
            &remaining[..take],
            length,
            declaration.kind,
            declaration.declaration,
            walk.counters_mut(),
        );
        let outcome = result.map_or_else(
            |outcome| shift_outcome(outcome, offset as u64),
            |_| Outcome::Unknown(Unknown::ParentScopeUnavailable),
        );
        let continue_container = declaration.container
            && matches!(outcome, Outcome::Unknown(Unknown::ParentScopeUnavailable))
            && take != 0;
        walk.record_outcome(&outcome);
        observations.push(project(&relative, &declaration, offset, take, outcome));
        offset += take;
        if !continue_container || offset == acquired.byte_length {
            break;
        }
    }
    Some(observations)
}

fn project(
    path: &str,
    file: &DeclaredFile,
    offset: usize,
    length: usize,
    outcome: Outcome,
) -> OfflineArtifactObservation {
    OfflineArtifactObservation::new(
        path,
        file.family.into(),
        PhysicalArtifactIdentity::new(format!(
            "unaddressed:{:?}:{}:{offset}",
            file.family, file.generation
        ))
        .unwrap(),
        PhysicalArtifactGeneration::encoded(file.generation).unwrap(),
        PhysicalByteRange::new(offset as u64, length as u64).ok(),
        outcome,
    )
}

fn classify(path: &str) -> Option<DeclaredFile> {
    use declarations::{free_space, root};
    let (family, kind, declaration, fixed_bytes, container, suffix) = if let Some(name) =
        path.strip_prefix("families/records/roots/root-")
    {
        (
            Family::RootRoutingBlock,
            8,
            root::ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
            None,
            false,
            tree_name(name)?,
        )
    } else if let Some(name) = path.strip_prefix("families/records/segment-manifests/segments-") {
        (
            Family::SegmentMembershipBlock,
            9,
            declarations::SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
            None,
            false,
            tree_name(name)?,
        )
    } else if let Some(name) = path.strip_prefix("families/records/free-space/free-space-") {
        if name.contains("-block-") {
            (
                Family::FreeSpaceMembershipBlock,
                10,
                free_space::FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
                None,
                false,
                tree_name(name)?,
            )
        } else {
            (
                Family::FreeSpaceHeader,
                7,
                free_space::FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
                Some(176),
                false,
                word(name.strip_suffix(".manifest")?)?,
            )
        }
    } else if let Some(name) = path.strip_prefix("families/records/extent-manifests/extent-") {
        (
            Family::ExtentManifest,
            6,
            declarations::EXTENT_MANIFEST_INTEGRITY_DECLARATION,
            Some(104),
            false,
            pair(name, ".manifest")?,
        )
    } else if let Some(name) = path.strip_prefix("families/records/segments/segment-") {
        (
            Family::PageFrame,
            3,
            declarations::PAGE_FRAME_INTEGRITY_DECLARATION,
            None,
            true,
            pair(name, ".pages")?,
        )
    } else if let Some(name) = path.strip_prefix("families/records/extents/extent-") {
        (
            Family::ExtentChunkFrame,
            4,
            declarations::EXTENT_CHUNK_INTEGRITY_DECLARATION,
            None,
            true,
            pair(name, ".data")?,
        )
    } else {
        return None;
    };
    Some(DeclaredFile {
        family,
        kind,
        generation: suffix,
        declaration,
        fixed_bytes,
        container,
    })
}
fn word(value: &str) -> Option<u64> {
    let number = u64::from_str_radix(value, 16).ok()?;
    (number != 0 && format!("{number:016x}") == value).then_some(number)
}
fn tree_name(name: &str) -> Option<u64> {
    let (generation, block) = name.strip_suffix(".manifest")?.split_once("-block-")?;
    word(block)?;
    word(generation)
}
fn pair(name: &str, suffix: &str) -> Option<u64> {
    let (identity, generation) = name.strip_suffix(suffix)?.split_once('-')?;
    word(identity)?;
    word(generation)
}
