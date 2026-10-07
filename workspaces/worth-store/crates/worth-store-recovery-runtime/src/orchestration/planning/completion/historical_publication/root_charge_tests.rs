use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::{charged_root, HistoricalFailure};
use crate::orchestration::planning::manifest_entry_budget::{
    manifest_entry_limit_for_test, EntryAdmission, ManifestEntryBudget,
};
use crate::orchestration::planning::page_observation::PageLimit;

const GENERATION: u64 = 5;

/// A historical root is charged its entry before it is read: with none
/// left, nothing is read and the refusal is the entry limit; with one left,
/// the root is read and that entry spent.
#[test]
fn a_historical_root_is_charged_its_entry_before_it_is_read() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("historical-root");
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    store_with_root(&root, format);
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(4, 1 << 16).unwrap();

    let mut none_left = ManifestEntryBudget::for_test(8, 8);
    let refused = charged_root(&mut discovery, &mut none_left, format, GENERATION).err();
    assert_eq!(
        refused,
        Some(HistoricalFailure::Limit(PageLimit::Entries(
            manifest_entry_limit_for_test(9, 8)
        )))
    );
    let counters = discovery.counters();
    assert_eq!(
        (counters.addressed_artifacts_read, counters.bytes_read),
        (0, 0)
    );

    let mut one_left = ManifestEntryBudget::for_test(8, 7);
    let (observed, _charge) =
        charged_root(&mut discovery, &mut one_left, format, GENERATION).unwrap();
    assert_eq!(observed.generation(), GENERATION);
    assert_eq!(one_left.remaining(), 0);
    assert!(discovery.counters().bytes_read > 0);
    discovery.finish();
}

fn store_with_root(root: &std::path::Path, format: PhysicalRecordFormatDeclaration) {
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("production media admission");
    };
    media.close();
    let roots = root.join("families/records/roots");
    std::fs::create_dir_all(&roots).unwrap();
    let artifact = RecordArtifactFile::RootManifest {
        generation: GENERATION,
    };
    let bytes = DurablePhysicalRootManifest::builder(GENERATION, 11, 4, 19)
        .admit()
        .unwrap()
        .encode(format);
    std::fs::write(roots.join(artifact.file_name()), bytes).unwrap();
}
