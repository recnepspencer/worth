use super::super::families::{
    extent::{read_extent_chunk, read_extent_manifest},
    free_space::{read_free_space_header, read_free_space_membership},
    page_frame::read_page_frame,
    root_routing::read_root_routing,
    segment_membership::read_segment_membership,
};
use super::super::{
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIntegrityOutcome as Outcome,
    OfflinePhysicalBlastRadius as Blast, OfflinePhysicalDamageCause as Cause,
    OfflinePhysicalDamageLocalization,
};
use super::ChildExpectation;
use worth_foundational::{
    PhysicalArtifactFamily as Family, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
    PhysicalByteRange,
};

pub(crate) fn inspect_expected(
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
        if super::super::crc32c::crc32c(&[bytes]) != checksum {
            return Err(damage(
                Cause::ChecksumMismatch,
                Some((0, bytes.len() as u64)),
                Blast::Artifact,
            ));
        }
    }
    Ok(children)
}

pub(crate) fn project(
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
            Outcome::Unsupported(super::super::OfflineUnsupportedPhysicalVersion::new(
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
