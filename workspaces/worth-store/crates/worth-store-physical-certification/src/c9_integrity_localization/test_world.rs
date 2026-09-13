use sha2::{Digest, Sha256};
use tempfile::TempDir;

use super::producer_fixture::materialize_canonical_root_fixture;
use super::*;

pub(super) struct RootFixture {
    pub(super) root: TempDir,
    pub(super) manifest: CleanRootArtifactManifest,
    pub(super) scenario: RootSliceScenario,
}

pub(super) fn root_fixture() -> RootFixture {
    let root = tempfile::tempdir().unwrap();
    let store_root = root.path().join("clean-store");
    std::fs::create_dir(&store_root).unwrap();
    let manifest = materialize_canonical_root_fixture(&store_root).unwrap();
    let reports = ExternalReportPaths::new(
        &store_root,
        root.path().join("reports/runtime.bin"),
        root.path().join("reports/offline.bin"),
    )
    .unwrap();
    let scenario = RootSliceScenario::new(store_root, reports);
    RootFixture {
        root,
        manifest,
        scenario,
    }
}

pub(super) fn fresh_row(
    fixture: &RootFixture,
    name: &str,
    counters: &mut RootLocalizationCounters,
) -> FreshRootArtifactRow {
    FreshRootArtifactRow::copy_from(
        &fixture.scenario,
        &fixture.manifest,
        fixture.root.path().join("rows").join(name),
        counters,
    )
    .unwrap()
}

pub(super) fn assert_clean_baseline_unchanged(fixture: &RootFixture) {
    for record in fixture.manifest.records() {
        let bytes = std::fs::read(
            fixture
                .scenario
                .clean_store_root()
                .join(record.relative_path()),
        )
        .unwrap();
        assert_eq!(Sha256::digest(bytes).as_slice(), record.content_sha256());
        let donor = std::fs::read(
            fixture
                .scenario
                .clean_store_root()
                .join(record.substitution_source_path()),
        )
        .unwrap();
        assert_eq!(
            Sha256::digest(donor).as_slice(),
            record.substitution_source_sha256()
        );
    }
    for (relative, expected_digest) in fixture.manifest.supporting_artifacts() {
        let bytes = std::fs::read(fixture.scenario.clean_store_root().join(relative)).unwrap();
        assert_eq!(Sha256::digest(bytes).as_slice(), expected_digest);
    }
}

pub(super) fn advance_expected_counters(
    expected: &mut RootLocalizationCounters,
    code: RootCorruptionCode,
    target_length: u64,
    clean_files: u64,
    clean_bytes: u64,
    after_files: u64,
    after_bytes: u64,
) {
    expected.isolated_world_copies += 1;
    expected.artifacts_opened += clean_files;
    expected.artifact_bytes_read += clean_bytes;
    expected.artifact_bytes_written += clean_bytes;
    expected.parent_oracle_derivations += 1;
    expected.artifacts_opened +=
        1 + u64::from(code == RootCorruptionCode::S) + clean_files + after_files;
    expected.artifact_bytes_read += target_length
        + if code == RootCorruptionCode::S {
            target_length
        } else {
            0
        }
        + clean_bytes
        + after_bytes;
    expected.artifact_bytes_written += match code {
        RootCorruptionCode::R => 0,
        RootCorruptionCode::T => target_length - 1,
        _ => target_length,
    };
    expected.checksum_refreshes += u64::from(matches!(
        code,
        RootCorruptionCode::L | RootCorruptionCode::P | RootCorruptionCode::U
    ));
    expected.namespace_removals += u64::from(code == RootCorruptionCode::R);
    expected.namespace_creations += u64::from(code == RootCorruptionCode::D);
    expected.editor_audits += 1;
}

pub(super) fn tree_statistics(root: &std::path::Path) -> (u64, u64) {
    let mut files = 0;
    let mut bytes = 0;
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                files += 1;
                bytes += metadata.len();
            } else {
                panic!("focused root world contains a non-file entry");
            }
        }
    }
    (files, bytes)
}
