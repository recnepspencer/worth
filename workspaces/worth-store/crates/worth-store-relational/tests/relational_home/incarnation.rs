//! Open identity, incarnation, pre-effect refusals and independent OS contention.
use super::{durable_run::open, workload};
use std::{fs, path::Path, process::Command};
use worth_execution::ExecutionRequest;
use worth_store_relational::{
    RelationalStoreHomeOpenRefusal as Refusal, RelationalStorePhysicalRefusalKind as Kind,
};

#[path = "incarnation/anchor_replay.rs"]
mod anchor_replay;
#[path = "incarnation/media.rs"]
mod media;
#[path = "incarnation/process.rs"]
mod process;

pub(crate) fn assert_contended(location: &Path) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "incarnation::contention_observer_child",
            "--nocapture",
        ])
        .env("WORTH_INCARNATION_OBSERVE", location)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "observer: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn contention_observer_child() {
    let Some(location) = std::env::var_os("WORTH_INCARNATION_OBSERVE") else {
        return;
    };
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    assert!(
        matches!(open(Path::new(&location), &execution), Err(Refusal::Physical(refusal)) if refusal.kind() == Kind::MutationOwnerContended)
    );
}

#[test]
fn home_identity_is_stable_and_each_successful_open_advances_once() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
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
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    open(&location, &execution).unwrap().close();
    let roots = location.join("families/records/roots");
    let mut files: Vec<_> = fs::read_dir(&roots)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    // Every schema-12 root becomes an unsupported schema-11 envelope. No
    // historical schema-11 payload is interpreted or migrated.
    for file in &files {
        let mut bytes = fs::read(file).unwrap();
        if bytes.get(9) == Some(&12) {
            bytes[9] = 11;
            fs::write(file, bytes).unwrap();
        }
    }
    let before: Vec<_> = files.iter().map(|f| fs::read(f).unwrap()).collect();
    let refused = open(&location, &execution);
    assert!(
        matches!(&refused, Err(Refusal::Physical(refusal)) if refusal.kind() == Kind::UnsupportedRootSchema { observed: 11 } && refusal.recovery_effects() == Some(0)),
        "{:?}",
        refused.map(|_| ())
    );
    assert_eq!(
        files
            .iter()
            .map(|f| fs::read(f).unwrap())
            .collect::<Vec<_>>(),
        before
    );
}

#[test]
fn retired_descriptor_versions_and_the_retired_anchor_refuse_without_advancing() {
    use worth_store::physical_runtime::{
        AnchoredRecordName, AnchoredRecordPublication, PhysicalMutationDeadline,
    };
    use worth_store_relational::RelationalStoreHomeDescriptorRefusal as Descriptor;
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
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
                anchor_replay::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ))
            .unwrap();
        home.close();
        assert!(matches!(
            open(&location, &execution),
            Err(Refusal::Descriptor(
                Descriptor::UnsupportedDescriptorVersion { observed: 1 }
            ))
        ));
    }
}

#[test]
fn missing_record_family_with_other_store_residue_is_never_initialized() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    open(&location, &execution).unwrap().close();
    // Remove only this test's disposable physical record family, leaving the
    // Store's other durable families. This models damaged existing media.
    fs::remove_dir_all(location.join("families/records")).unwrap();
    fs::remove_dir_all(location.join("staging/records")).unwrap();
    let residue = location.join("families/interrupted-owner");
    fs::create_dir_all(&residue).unwrap();
    fs::write(residue.join("truth"), b"retained").unwrap();
    assert!(open(&location, &execution).is_err());
    assert!(!location.join("families/records").exists());
    assert_eq!(fs::read(residue.join("truth")).unwrap(), b"retained");
}

#[test]
fn missing_record_family_with_only_namespace_identity_is_never_initialized() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    open(&location, &execution).unwrap().close();
    let identity = fs::read(location.join("namespace/identity")).unwrap();
    // Lose all family data in this test's disposable home, including its
    // physical-work families. Identity alone cannot prove that the previous
    // open stopped during first initialization.
    for directory in ["families", "staging"] {
        let directory = location.join(directory);
        fs::remove_dir_all(&directory).unwrap();
        fs::create_dir(&directory).unwrap();
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
    }
    assert!(open(&location, &execution).is_err());
    assert!(!location.join("families/records").exists());
    assert_eq!(
        fs::read(location.join("namespace/identity")).unwrap(),
        identity
    );
}
