//! Open identity, incarnation, pre-effect refusals and independent OS contention.
#[allow(unused_imports)] use crate::incarnation::*;
use {crate::durable_run::open, crate::workload};
use {std::fs, std::path::Path, std::process::Command};
use {worth_execution::ExecutionRequest};
use {worth_store_relational::RelationalStoreHomeOpenRefusal as Refusal, worth_store_relational::RelationalStorePhysicalRefusalKind as Kind};

#[test]
fn contention_observer_child() {
    let Some(location) = std::env::var_os("WORTH_INCARNATION_OBSERVE") else {
        return;
    };
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    assert!(
        matches!(open(std::path::Path::new(&location), &execution), Err(worth_store_relational::RelationalStoreHomeOpenRefusal::Physical(refusal)) if refusal.kind() == worth_store_relational::RelationalStorePhysicalRefusalKind::MutationOwnerContended)
    );
}

#[test]
fn home_identity_is_stable_and_each_successful_open_advances_once() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let first = open(&location, &execution).unwrap();
    assert_eq!(first.incarnation(), 1);
    let identity = first.home_identity();
    first.close();
    for expected in 2..=4 {
        let home = open(&location, &execution).unwrap();
        assert_eq!(home.home_identity(), identity);
        assert_eq!(home.incarnation(), expected);
        home.close();
    }
}

#[test]
fn schema_eleven_refuses_before_any_media_effect() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    open(&location, &execution).unwrap().close();
    let roots = location.join("families/records/roots");
    let mut files: Vec<_> = std::fs::read_dir(&roots)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    // Every schema-12 root becomes an unsupported schema-11 envelope. No
    // historical schema-11 payload is interpreted or migrated.
    for file in &files {
        let mut bytes = std::fs::read(file).unwrap();
        if bytes.get(9) == Some(&12) {
            bytes[9] = 11;
            std::fs::write(file, bytes).unwrap();
        }
    }
    let before: Vec<_> = files.iter().map(|f| std::fs::read(f).unwrap()).collect();
    let refused = open(&location, &execution);
    assert!(
        matches!(&refused, Err(worth_store_relational::RelationalStoreHomeOpenRefusal::Physical(refusal)) if refusal.kind() == worth_store_relational::RelationalStorePhysicalRefusalKind::UnsupportedRootSchema { observed: 11 } && refusal.recovery_effects() == Some(0)),
        "{:?}",
        refused.map(|_| ())
    );
    assert_eq!(
        files
            .iter()
            .map(|f| std::fs::read(f).unwrap())
            .collect::<Vec<_>>(),
        before
    );
}

#[test]
fn retired_descriptor_versions_and_the_retired_anchor_refuse_without_advancing() {
    use {worth_store::physical_runtime::AnchoredRecordName, worth_store::physical_runtime::AnchoredRecordPublication, worth_store::physical_runtime::PhysicalMutationDeadline};
    use {worth_store_relational::RelationalStoreHomeDescriptorRefusal as Descriptor};
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    for retired_anchor in [false, true] {
        let parent = tempfile::tempdir().unwrap();
        let location = parent.path().join("home");
        let home = open(&location, &execution).unwrap();
        let records = home.certification_serving().anchored_records().unwrap();
        let selected = records
            .read(AnchoredRecordName::new("home").unwrap(), 65536)
            .unwrap()
            .unwrap();
        let mut bytes = selected.bytes().to_vec();
        bytes[8..10].copy_from_slice(&1_u16.to_le_bytes());
        let (name, expected) = if retired_anchor {
            ("relational.home", 0)
        } else {
            ("home", selected.version().generation())
        };
        records
            .publish(AnchoredRecordPublication::new(
                AnchoredRecordName::new(name).unwrap(),
                expected,
                bytes,
                crate::incarnation::anchor_replay::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ))
            .unwrap();
        home.close();
        assert!(matches!(
            open(&location, &execution),
            Err(worth_store_relational::RelationalStoreHomeOpenRefusal::Descriptor(
                Descriptor::UnsupportedDescriptorVersion { observed: 1 }
            ))
        ));
    }
}

#[test]
fn missing_record_family_with_other_store_residue_is_never_initialized() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    open(&location, &execution).unwrap().close();
    // Remove only this test's disposable physical record family, leaving the
    // Store's other durable families. This models damaged existing media.
    std::fs::remove_dir_all(location.join("families/records")).unwrap();
    std::fs::remove_dir_all(location.join("staging/records")).unwrap();
    let residue = location.join("families/interrupted-owner");
    std::fs::create_dir_all(&residue).unwrap();
    std::fs::write(residue.join("truth"), b"retained").unwrap();
    assert!(open(&location, &execution).is_err());
    assert!(!location.join("families/records").exists());
    assert_eq!(std::fs::read(residue.join("truth")).unwrap(), b"retained");
}

#[test]
fn missing_record_family_with_only_namespace_identity_is_never_initialized() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    open(&location, &execution).unwrap().close();
    let identity = std::fs::read(location.join("namespace/identity")).unwrap();
    // Lose all family data in this test's disposable home, including its
    // physical-work families. Identity alone cannot prove that the previous
    // open stopped during first initialization.
    for directory in ["families", "staging"] {
        let directory = location.join(directory);
        std::fs::remove_dir_all(&directory).unwrap();
        std::fs::create_dir(&directory).unwrap();
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
    }
    assert!(open(&location, &execution).is_err());
    assert!(!location.join("families/records").exists());
    assert_eq!(
        std::fs::read(location.join("namespace/identity")).unwrap(),
        identity
    );
}
#[path = "anchor_replay/mod.rs"]
pub(crate) mod anchor_replay;
#[path = "media/mod.rs"]
pub(crate) mod media;
#[path = "process/mod.rs"]
pub(crate) mod process;
