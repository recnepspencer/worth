use super::test_world::root_fixture;
use super::{ExternalReportPathDenial, ExternalReportPaths};

#[test]
fn report_paths_are_external_canonical_and_symlink_safe() {
    let fixture = root_fixture();
    assert!(!fixture
        .scenario
        .reports()
        .runtime()
        .starts_with(fixture.scenario.clean_store_root()));
    assert!(!fixture
        .scenario
        .reports()
        .offline()
        .starts_with(fixture.scenario.clean_store_root()));
    assert!(ExternalReportPaths::new(
        fixture.scenario.clean_store_root(),
        fixture.scenario.clean_store_root().join("runtime.bin"),
        fixture.root.path().join("outside.bin"),
    )
    .is_err());
    assert_eq!(
        ExternalReportPaths::new(
            fixture.scenario.clean_store_root(),
            fixture
                .root
                .path()
                .join("reports/../clean-store/runtime.bin"),
            fixture.root.path().join("reports/offline.bin"),
        ),
        Err(ExternalReportPathDenial::ParentTraversal)
    );

    let outside_root = fixture.root.path().join("physical-reports");
    std::fs::create_dir(&outside_root).unwrap();
    let resolved = ExternalReportPaths::new(
        fixture.scenario.clean_store_root(),
        outside_root.join("nested/runtime.bin"),
        outside_root.join("offline.bin"),
    )
    .unwrap();
    let canonical_outside = std::fs::canonicalize(&outside_root).unwrap();
    assert!(resolved.runtime().starts_with(&canonical_outside));

    let authorized = ExternalReportPaths::authorize(
        fixture.scenario.clean_store_root(),
        outside_root.join("subject.report"),
    )
    .unwrap();
    assert!(authorized.as_path().starts_with(&canonical_outside));
    assert_eq!(
        ExternalReportPaths::authorize(
            fixture.scenario.clean_store_root(),
            fixture.scenario.clean_store_root().join("subject.report"),
        ),
        Err(ExternalReportPathDenial::InsideStoreRoot)
    );

    assert_report_alias_is_rejected_when_supported(&fixture);
}

fn assert_report_alias_is_rejected_when_supported(fixture: &super::test_world::RootFixture) {
    let alias = fixture.root.path().join("store-alias");
    if create_directory_alias(fixture.scenario.clean_store_root(), &alias).is_err() {
        return;
    }
    assert_eq!(
        ExternalReportPaths::new(
            fixture.scenario.clean_store_root(),
            alias.join("runtime.bin"),
            fixture.root.path().join("alias-test-offline.bin"),
        ),
        Err(ExternalReportPathDenial::InsideStoreRoot)
    );
}

#[cfg(windows)]
fn create_directory_alias(
    target: &std::path::Path,
    alias: &std::path::Path,
) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, alias)
}

#[cfg(unix)]
fn create_directory_alias(
    target: &std::path::Path,
    alias: &std::path::Path,
) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, alias)
}
