use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, FilesystemObservationBound,
    PhysicalRuntimeAdmission, PhysicalStore, QualifiedRecoveryFilesystemMedia,
    RecoveryDiscoveryFailure,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::read;
use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::progression::PlanningResidentAllowance;

const GENERATION: u64 = 7;

#[test]
fn absent_optional_root_reads_with_zero_resident_space() {
    let fixture = Fixture::new("successor-root-absent", false);
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&fixture.root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(2, 4096).unwrap();
    let mut resident = PlanningResidentAllowance::new(0, 0).unwrap();
    let artifact = RecordArtifactFile::RootManifest {
        generation: GENERATION,
    };
    let observed = read(&mut discovery, artifact, format(), &mut resident).unwrap();
    assert!(observed.bytes().is_none());
    assert_eq!(resident.used(), 0);
    assert_eq!(discovery.counters().bytes_read, 0);
    discovery.finish();
}

#[test]
fn present_root_reports_exact_resident_crossing_without_reading_bytes() {
    let fixture = Fixture::new("successor-root-resident", true);
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&fixture.root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(2, 4096).unwrap();
    let admitted = fixture.bytes.len() as u64 - 1;
    let mut resident = PlanningResidentAllowance::new(0, admitted).unwrap();
    let artifact = RecordArtifactFile::RootManifest {
        generation: GENERATION,
    };
    let denial = read(&mut discovery, artifact, format(), &mut resident).unwrap_err();
    assert!(matches!(
        denial,
        PhysicalRecoverySuccessorCandidateDenial::RecoveryMemoryBytes {
            artifact: RecordArtifactFile::RootManifest { generation: GENERATION },
            generation: GENERATION,
            observed,
            admitted: limit,
        } if observed == fixture.bytes.len() as u64 && limit == admitted
    ));
    assert_eq!(discovery.counters().bytes_read, 0);
    discovery.finish();
}

#[test]
fn cumulative_discovery_limit_remains_a_discovery_denial() {
    let fixture = Fixture::new("successor-root-observation", true);
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&fixture.root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let admitted = fixture.bytes.len() as u64 - 1;
    let mut discovery = media.bounded_discovery(2, admitted).unwrap();
    let mut resident = PlanningResidentAllowance::new(0, 4096).unwrap();
    let artifact = RecordArtifactFile::RootManifest {
        generation: GENERATION,
    };
    let denial = read(&mut discovery, artifact, format(), &mut resident).unwrap_err();
    let PhysicalRecoverySuccessorCandidateDenial::Discovery {
        artifact: RecordArtifactFile::RootManifest {
            generation: GENERATION,
        },
        failure: RecoveryDiscoveryFailure::Limit(past),
        ..
    } = denial
    else {
        panic!("the reader's own bytes ran out: {denial:?}");
    };
    assert_eq!(
        (past.dimension(), past.observed(), past.admitted()),
        (
            FilesystemObservationBound::ObservationBytes,
            admitted + 1,
            admitted
        )
    );
    assert_eq!(resident.used(), 0);
    discovery.finish();
}

/// A root manifest is one page of its format at most. A longer one is
/// damage, however little resident space the candidate has left.
#[test]
fn a_root_past_its_page_is_damage_whatever_resident_space_is_left() {
    let fixture = Fixture::new("successor-root-oversized", true);
    let page = u64::from(format().page_size().bytes());
    let artifact = RecordArtifactFile::RootManifest {
        generation: GENERATION,
    };
    let path = fixture
        .root
        .join("families/records/roots")
        .join(artifact.file_name());
    std::fs::write(path, vec![0_u8; page as usize + 1]).unwrap();
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&fixture.root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(4, 4 * page).unwrap();
    for resident_space in [16, page, 2 * page] {
        let mut resident = PlanningResidentAllowance::new(0, resident_space).unwrap();
        let denial = read(&mut discovery, artifact, format(), &mut resident).unwrap_err();
        let past = match &denial {
            PhysicalRecoverySuccessorCandidateDenial::Discovery {
                failure: RecoveryDiscoveryFailure::Limit(past),
                ..
            } => Some((past.dimension(), past.observed(), past.admitted())),
            _ => None,
        };
        // The reader names the read it refused: one asked for the page or
        // the resident space left, whichever is smaller.
        assert_eq!(
            past,
            Some((
                FilesystemObservationBound::RequestedBytes,
                page + 1,
                resident_space.min(page)
            )),
            "{resident_space} resident bytes: {denial:?}",
        );
        assert_eq!(resident.used(), 0);
    }
    assert_eq!(discovery.counters().bytes_read, 0);
    discovery.finish();
}

#[test]
fn sufficient_resident_space_retains_exact_canonical_root_bytes() {
    let fixture = Fixture::new("successor-root-adequate", true);
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&fixture.root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(2, 4096).unwrap();
    let admitted = fixture.bytes.len() as u64;
    let mut resident = PlanningResidentAllowance::new(0, admitted).unwrap();
    let artifact = RecordArtifactFile::RootManifest {
        generation: GENERATION,
    };
    let observed = read(&mut discovery, artifact, format(), &mut resident).unwrap();
    assert_eq!(observed.bytes(), Some(fixture.bytes.as_slice()));
    assert_eq!(observed.owned_heap_bytes(), Some(admitted));
    assert_eq!(resident.used(), admitted);
    discovery.finish();
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

struct Fixture {
    root: std::path::PathBuf,
    bytes: Vec<u8>,
    _directory: tempfile::TempDir,
}

impl Fixture {
    fn new(label: &str, present: bool) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join(label);
        let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
        let TransitionOutcome::Success(media) = runtime
            .try_admit_filesystem_media(FilesystemMediaAdmission::production(
                FilesystemAccessPosture::CoordinatedServiceAccount,
            ))
            .into_raw()
        else {
            panic!("production media admission");
        };
        media.close();
        let bytes = DurablePhysicalRootManifest::builder(GENERATION, 11, 4, 19)
            .admit()
            .unwrap()
            .encode(format());
        if present {
            let roots = root.join("families/records/roots");
            std::fs::create_dir_all(&roots).unwrap();
            std::fs::write(
                roots.join(
                    RecordArtifactFile::RootManifest {
                        generation: GENERATION,
                    }
                    .file_name(),
                ),
                &bytes,
            )
            .unwrap();
        }
        Self {
            root,
            bytes,
            _directory: directory,
        }
    }
}
