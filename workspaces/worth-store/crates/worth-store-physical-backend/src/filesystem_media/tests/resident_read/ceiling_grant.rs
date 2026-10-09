//! Every whole-artifact reader decides its bounds with the artifact's real
//! length, in one order: past its ceiling is damage, then past the caller's
//! grant is that budget's, then past the observation's bytes is the
//! observation's. Each case below is read by every whole reader.

use super::record_storage::storage;
use super::*;
use crate::recovery_media::grant_for_test::TestBytes;
use crate::recovery_media::FixedArtifact;
use worth_store_physical_format::ROOT_SELECTOR_BYTES;

const ROOT: RecordArtifactFile = RecordArtifactFile::RootManifest { generation: 6 };

/// What one reader's grant is: none, or bytes of a test budget.
#[derive(Clone, Copy)]
enum Granted {
    Ceiling,
    Bytes(u64),
}

/// Where each whole reader stopped reading `length` bytes at `artifact` under
/// `ceiling`, `granted` and `reader` observation bytes; each on its own
/// observation.
fn whole_stops(
    artifact: RecordArtifactFile,
    ceiling: impl Fn() -> ArtifactCeiling,
    length: u64,
    granted: Granted,
    reader: u64,
) -> Vec<Option<Stop>> {
    let bytes = vec![7; usize::try_from(length).unwrap()];
    let directory = match artifact {
        RecordArtifactFile::RootManifest { .. } => "families/records/roots",
        _ => "families/records",
    };
    let observe = || {
        discovery(
            |root| {
                std::fs::create_dir_all(root.join(directory)).unwrap();
                std::fs::write(root.join(directory).join(artifact.file_name()), &bytes).unwrap();
            },
            4,
            reader,
        )
    };
    let allocate = |length| Ok::<_, DeniedAllocation>(vec![0; length]);
    let mut stops = Vec::new();
    let (_parent, _observer, mut addressed) = observe();
    let (_parent_allocated, _observer, mut allocated) = observe();
    let (_parent_stored, _observer, mut stored) = observe();
    match granted {
        Granted::Ceiling => {
            stops.push(stopped(&addressed.read(ceiling(), uncharged())));
            stops.push(stopped(&allocated.read_with_allocator(
                ceiling(),
                uncharged(),
                allocate,
            )));
            stops.push(stopped(&stored.read_with_storage(
                ceiling(),
                uncharged(),
                &mut storage(None, false),
            )));
        }
        Granted::Bytes(bytes) => {
            stops.push(stopped(&addressed.read(ceiling(), grant(bytes))));
            stops.push(stopped(&allocated.read_with_allocator(
                ceiling(),
                grant(bytes),
                allocate,
            )));
            stops.push(stopped(&stored.read_with_storage(
                ceiling(),
                grant(bytes),
                &mut storage(None, false),
            )));
        }
    }
    // A path storage this platform never qualifies addresses nothing.
    if cfg!(not(windows)) {
        assert_eq!(stops.pop(), Some(Some(Stop::InvalidAddress)));
    }
    stops
}

fn root(length: u64, granted: Granted, reader: u64) -> Vec<Option<Stop>> {
    whole_stops(ROOT, || root_ceiling(6), length, granted, reader)
}

fn each(stop: Option<Stop>) -> Vec<Option<Stop>> {
    vec![stop; if cfg!(windows) { 3 } else { 2 }]
}

#[test]
fn a_whole_artifact_exactly_at_its_ceiling_reads() {
    let page = page();
    assert_eq!(root(page, Granted::Ceiling, page), each(None));
    assert_eq!(root(page, Granted::Bytes(page), page), each(None));
}

/// One byte past the ceiling, and five: each names the real length, which
/// the artifact's metadata reports, not one past what the tree was asked.
#[test]
fn a_whole_artifact_past_its_ceiling_is_damage_with_its_real_length() {
    let page = page();
    for length in [page + 1, page + 5] {
        let past = Some(Stop::PastCeiling {
            length,
            ceiling: page,
        });
        assert_eq!(root(length, Granted::Ceiling, 4 * page), each(past.clone()));
        // However little the grant or the observation holds, a length past
        // the ceiling is damage first.
        assert_eq!(
            root(length, Granted::Bytes(1), 4 * page),
            each(past.clone())
        );
        assert_eq!(root(length, Granted::Ceiling, 1), each(past));
    }
}

/// A file exactly at its ceiling under a smaller grant passed the grant, not
/// the ceiling.
#[test]
fn a_whole_artifact_at_its_ceiling_past_its_grant_is_the_grants() {
    let page = page();
    assert_eq!(
        root(page, Granted::Bytes(page - 1), page),
        each(Some(Stop::PastGrant {
            granted: page - 1,
            length: page
        })),
    );
}

#[test]
fn a_whole_artifact_exactly_at_its_grant_reads_and_one_past_names_its_length() {
    assert_eq!(root(10, Granted::Bytes(10), 32), each(None));
    for length in [11, 13] {
        assert_eq!(
            root(length, Granted::Bytes(10), 32),
            each(Some(Stop::PastGrant {
                granted: 10,
                length
            })),
        );
    }
}

/// A length the grant admits, refused by the observation's bytes, is the
/// observation's limit with the backend's own counts.
#[test]
fn a_whole_artifact_within_its_grant_past_the_observation_is_the_observations() {
    use FilesystemObservationBound::ObservationBytes;
    assert_eq!(
        root(10, Granted::Bytes(10), 9),
        each(Some(Stop::Observation(ObservationBytes, 10, 9))),
    );
    assert_eq!(
        root(7, Granted::Ceiling, 4),
        each(Some(Stop::Observation(ObservationBytes, 7, 4))),
    );
    // An observation admitted no bytes refuses any read with its real length.
    assert_eq!(
        root(7, Granted::Ceiling, 0),
        each(Some(Stop::Observation(ObservationBytes, 7, 0))),
    );
    assert_eq!(root(4, Granted::Ceiling, 4), each(None));
}

#[test]
fn a_fixed_slot_is_bounded_by_its_format_size() {
    let slot = RecordArtifactFile::CurrentRootSelector;
    let selector = || ArtifactCeiling::fixed(FixedArtifact::CurrentRootSelector);
    let size = ROOT_SELECTOR_BYTES as u64;
    assert_eq!(
        whole_stops(slot, selector, size, Granted::Ceiling, 4 * size),
        each(None)
    );
    assert_eq!(
        whole_stops(slot, selector, size + 1, Granted::Ceiling, 4 * size),
        each(Some(Stop::PastCeiling {
            length: size + 1,
            ceiling: size
        })),
    );
    assert_eq!(
        whole_stops(slot, selector, size, Granted::Bytes(size - 1), 4 * size),
        each(Some(Stop::PastGrant {
            granted: size - 1,
            length: size
        })),
    );
    let (_parent, _observer, mut discovery) = discovery(
        |root| {
            std::fs::create_dir_all(root.join("families/records")).unwrap();
            std::fs::write(
                root.join("families/records").join(slot.file_name()),
                vec![0; ROOT_SELECTOR_BYTES],
            )
            .unwrap();
        },
        4,
        4 * size,
    );
    discovery.read(selector(), uncharged()).observed().unwrap();
    assert_eq!(discovery.counters().fixed_slots_read, 1);
    assert_eq!(discovery.counters().addressed_artifacts_read, 0);
}

/// A segment frame is exactly one page at its frame's offset; a segment too
/// short for it is the tree's damage.
#[test]
fn a_segment_frame_is_one_page_at_its_offset() {
    let page = page();
    let segment = RecordArtifactFile::Segment {
        segment: 4,
        generation: 1,
    };
    let frame = |frame| {
        ArtifactCeiling::page(
            format(),
            PageAddress::SegmentFrame {
                segment: 4,
                generation: 1,
                frame,
            },
        )
    };
    let mut bytes = vec![1; usize::try_from(page).unwrap()];
    bytes.extend(vec![2; usize::try_from(page).unwrap()]);
    let (_parent, _observer, mut discovery) = discovery(
        |root| {
            let directory = root.join("families/records/segments");
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join(segment.file_name()), &bytes).unwrap();
        },
        4,
        8 * page,
    );
    let second = discovery.read(frame(1), uncharged()).observed().unwrap();
    assert_eq!(second.offset(), page);
    assert_eq!(
        second.bytes(),
        Some(&bytes[usize::try_from(page).unwrap()..])
    );
    assert_eq!(
        stopped(&discovery.read(frame(1), grant(page - 1))),
        Some(Stop::PastGrant {
            granted: page - 1,
            length: page
        })
    );
    assert_eq!(
        stopped(&discovery.read(frame(2), uncharged())),
        Some(Stop::Media(ArtifactTreeFailureKind::Damaged))
    );
}

/// The grant a test budget mints carries its own dimension back.
#[test]
fn a_past_grant_names_its_budget() {
    let (_parent, _observer, mut discovery) =
        discovery(|root| write_root_artifact(root, ROOT, b"present"), 4, 32);
    match discovery.read(root_ceiling(6), grant(3)) {
        TransitionOutcome::Denied(ReadRefusal::PastGrant(overrun)) => {
            assert_eq!(overrun.dimension(), TestBytes);
        }
        other => panic!("past the grant: {other:?}"),
    }
}
