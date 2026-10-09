//! A read budget grants what is left, charges what a read returned, and
//! refuses a read past its grant with the budget's own counts.

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, ArtifactCeiling, FilesystemAccessPosture,
    FilesystemMediaAdmission, GrantedRead, GrantedReadStop, PageAddress, PhysicalRuntimeAdmission,
    PhysicalStore, QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::RecoveryReadBudget;
use crate::entry::{PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension};

const GENERATION: u64 = 7;

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

/// A store holding one root manifest, and that manifest's length.
fn store_with_one_root() -> (tempfile::TempDir, AdmittedRecoveryFilesystemMedia, u64) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
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
    let roots = root.join("families/records/roots");
    std::fs::create_dir_all(&roots).unwrap();
    let file = RecordArtifactFile::RootManifest {
        generation: GENERATION,
    };
    std::fs::write(roots.join(file.file_name()), &bytes).unwrap();
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    (directory, media, bytes.len() as u64)
}

/// A manifest byte budget of `bytes`.
fn manifest_bytes(bytes: u64) -> RecoveryReadBudget {
    let mut values = [1 << 20; 19];
    values[2] = bytes;
    RecoveryReadBudget::declared(
        &PhysicalRecoveryLimitDeclaration::from_values_for_test(values),
        PhysicalRecoveryLimitDimension::ManifestBytes,
    )
}

fn ceiling() -> ArtifactCeiling {
    ArtifactCeiling::page(
        format(),
        PageAddress::RootManifest {
            generation: GENERATION,
        },
    )
}

#[test]
fn each_read_is_granted_what_is_left_and_charged_what_it_returned() {
    let (_directory, media, length) = store_with_one_root();
    let mut discovery = media.bounded_discovery(4, 1 << 20).unwrap();
    // Two reads of the root fit; a third needs one byte more than is left.
    let mut budget = manifest_bytes(3 * length - 1);
    for read in 0..2 {
        assert_eq!(budget.grant().bytes(), Some(3 * length - 1 - read * length));
        let observed = discovery.read(ceiling(), budget.grant()).granted().unwrap();
        budget.charge(&observed);
        assert_eq!(budget.spent(), (read + 1) * length);
    }
    let Err(GrantedReadStop::PastGrant(overrun)) =
        discovery.read(ceiling(), budget.grant()).granted()
    else {
        panic!("the third read passes what is left");
    };
    assert_eq!((overrun.granted(), overrun.length()), (length - 1, length));
    // The limit counts what the budget spent beside the grant, plus the
    // read's real length.
    let limit = budget.refuse(overrun).unwrap();
    assert_eq!(
        (limit.dimension(), limit.observed(), limit.admitted()),
        (
            PhysicalRecoveryLimitDimension::ManifestBytes,
            3 * length,
            3 * length - 1
        ),
    );
    // The refused read was not charged.
    assert_eq!(budget.spent(), 2 * length);
    discovery.finish();
}

#[test]
fn an_empty_budget_refuses_its_first_read_with_that_reads_length() {
    let (_directory, media, length) = store_with_one_root();
    let mut discovery = media.bounded_discovery(4, 1 << 20).unwrap();
    let budget = manifest_bytes(0);
    let Err(GrantedReadStop::PastGrant(overrun)) =
        discovery.read(ceiling(), budget.grant()).granted()
    else {
        panic!("an empty budget grants nothing");
    };
    let limit = budget.refuse(overrun).unwrap();
    assert_eq!((limit.observed(), limit.admitted()), (length, 0));
    assert_eq!(discovery.counters().bytes_read, 0);
    discovery.finish();
}
