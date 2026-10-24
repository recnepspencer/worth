//! Open refusals: a home that is not this runtime's, a location holding
//! something other than a Store, and a home whose checkpoint or journal is
//! damaged all refuse before service, with a typed refusal, and none of them
//! is treated as an empty home.

use std::fs;
use std::path::{Path, PathBuf};

use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, AnchoredRecordName,
    AnchoredRecordPublication, CanonicalCommitPublication, ManifestEntryCapacity,
    PhysicalMutationDeadline, PhysicalRecordFormatDeclaration, PhysicalRecordPlacementPolicy,
};
use worth_store_relational::{
    RelationalStoreHome, RelationalStoreHomeOpenRefusal, RelationalStoreHomeRequest,
    RelationalStoreJournalRefusal,
};

use super::durable_run::{apply, assert_holds, open, steps_before};
use super::schema_transition;
use super::selected_checkpoint;
use super::workload::{self, Script};

const SEED: u64 = 0x5eed_0501;
const COMMITS: usize = 6;
const CHECKPOINT_MAGIC: &[u8] = b"WSRCKPT1";
const ARTIFACT_MAGIC: &[u8] = b"WRTHRCCA";

/// A closed home holding `commits` commits of the script, with a checkpoint
/// after the third.
fn home_with(
    commits: usize,
    execution: &ExecutionRequest<'_, '_>,
) -> (tempfile::TempDir, PathBuf, Script, Vec<Vec<u8>>) {
    let script = Script::generate(SEED, COMMITS, 3);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, execution).expect("a new home opens");
    apply(&home, execution, steps_before(&script, commits));
    let acknowledged = workload::artifacts_of(home.runtime());
    home.close();
    (directory, location, script, acknowledged)
}

#[test]
fn a_home_opened_as_another_runtime_refuses_as_foreign() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (_directory, location, script, acknowledged) = home_with(2, &execution);
    let foreign = RelationalRuntimeApi::builder()
        .runtime_setup(|runtime| {
            runtime.runtime_name("another-runtime");
        })
        .schema_setup(|setup| {
            setup.schema_registry(schema_transition::registry(schema_transition::SOURCE));
        });
    let refused = RelationalStoreHome::open(
        RelationalStoreHomeRequest::new(&location, foreign),
        &execution,
    );
    assert!(
        matches!(
            refused,
            Err(RelationalStoreHomeOpenRefusal::ForeignHome { .. })
        ),
        "{:?}",
        refused.map(|_| ())
    );
    let home = open(&location, &execution).expect("the home is still its runtime's");
    assert_holds(&home, &script, &acknowledged);
    home.close();
}

#[test]
fn a_location_holding_something_else_is_refused_not_initialized_over() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    fs::create_dir_all(&location).unwrap();
    fs::write(location.join("notes.txt"), b"not a store").unwrap();
    let refused = open(&location, &execution);
    assert!(
        matches!(refused, Err(RelationalStoreHomeOpenRefusal::Physical(_))),
        "{:?}",
        refused.map(|_| ())
    );
    let entries: Vec<_> = fs::read_dir(&location)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(
        entries,
        ["notes.txt"],
        "the refusal wrote into the location"
    );
    assert_eq!(
        fs::read(location.join("notes.txt")).unwrap(),
        b"not a store"
    );
}

/// Flip one byte `offset` bytes after every occurrence of `pattern` in every
/// file under `root`. Returns how many bytes were flipped.
fn flip_after(
    root: &Path,
    pattern: &[u8],
    offset: usize,
    matches: &dyn Fn(&[u8]) -> bool,
) -> usize {
    let mut flipped = 0;
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            flipped += flip_after(&path, pattern, offset, matches);
            continue;
        }
        let mut bytes = fs::read(&path).unwrap();
        let mut changed = false;
        let mut start = 0;
        while let Some(found) = bytes[start..]
            .windows(pattern.len())
            .position(|window| window == pattern)
        {
            let at = start + found;
            if at + offset < bytes.len() && matches(&bytes[at..]) {
                bytes[at + offset] ^= 0x5a;
                changed = true;
                flipped += 1;
            }
            start = at + pattern.len();
        }
        if changed {
            fs::write(&path, bytes).unwrap();
        }
    }
    flipped
}

fn assert_refuses_before_service(location: &Path, case: &str) -> RelationalStoreHomeOpenRefusal {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    match open(location, &execution) {
        Ok(home) => panic!(
            "{case}: a damaged home served {} commits",
            workload::artifacts_of(home.runtime()).len()
        ),
        Err(refusal) => refusal,
    }
}

#[test]
fn a_damaged_checkpoint_in_the_store_files_refuses_before_service() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (_directory, location, _, _) = home_with(4, &execution);
    let flipped = flip_after(&location, CHECKPOINT_MAGIC, 256, &|_| true);
    assert!(
        flipped > 0,
        "the checkpoint record was found in the Store's files"
    );
    let refusal = assert_refuses_before_service(&location, "damaged checkpoint");
    assert!(
        matches!(
            refusal,
            RelationalStoreHomeOpenRefusal::Checkpoint(_)
                | RelationalStoreHomeOpenRefusal::Physical(_)
        ),
        "{refusal:?}"
    );
}

#[test]
fn a_damaged_journal_entry_in_the_store_files_refuses_before_service() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (_directory, location, _, _) = home_with(5, &execution);
    // Damage the artifact at position 5, the journal tail above the
    // checkpoint, in every copy Store holds.
    let at_position_five = |frame: &[u8]| frame.get(10..18) == Some(&5_u64.to_le_bytes()[..]);
    let flipped = flip_after(&location, ARTIFACT_MAGIC, 512, &at_position_five);
    assert!(
        flipped > 0,
        "the journal entry was found in the Store's files"
    );
    let refusal = assert_refuses_before_service(&location, "damaged journal entry");
    assert!(
        matches!(
            refusal,
            RelationalStoreHomeOpenRefusal::Journal(_)
                | RelationalStoreHomeOpenRefusal::Physical(_)
        ),
        "{refusal:?}"
    );
}

/// The placement a home publishes under: the default record format with a
/// manifest capacity of 64.
fn home_placement() -> AdmittedRecordPlacementPolicy {
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap()
}

fn deadline() -> PhysicalMutationDeadline {
    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
}

#[test]
fn a_checkpoint_record_store_accepts_but_relational_cannot_readmit_refuses() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (_directory, location, _, _) = home_with(4, &execution);
    let home = open(&location, &execution).expect("the home opens");
    let generation = selected_checkpoint::generation(&home);
    let records = home.certification_serving().anchored_records().unwrap();
    let name = AnchoredRecordName::new(selected_checkpoint::ANCHOR).unwrap();
    let mut payload = records
        .read(name, 1 << 30)
        .unwrap()
        .unwrap()
        .into_bytes()
        .to_vec();
    // Keep the home's header (magic, version, covered position); damage the
    // native checkpoint inside it.
    let middle = 18 + (payload.len() - 18) / 2;
    payload[middle] ^= 0x5a;
    let placement = home_placement();
    records
        .publish(AnchoredRecordPublication::new(
            name,
            generation,
            payload,
            placement,
            deadline(),
        ))
        .expect("Store accepts the record");
    home.close();
    let refusal = assert_refuses_before_service(&location, "unreadmittable checkpoint");
    assert!(
        matches!(refusal, RelationalStoreHomeOpenRefusal::Readmission(_)),
        "{refusal:?}"
    );
}

#[test]
fn a_journal_entry_store_accepts_but_relational_cannot_readmit_refuses() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (_directory, location, _, acknowledged) = home_with(5, &execution);
    let home = open(&location, &execution).expect("the home opens");
    let mut bytes = acknowledged[4].clone();
    bytes[ARTIFACT_MAGIC.len() + 600] ^= 0x5a;
    home.certification_serving()
        .canonical_commits()
        .unwrap()
        .publish(CanonicalCommitPublication::new(
            6,
            bytes,
            home_placement(),
            deadline(),
        ))
        .expect("Store accepts the entry");
    home.close();
    let refusal = assert_refuses_before_service(&location, "unreadmittable journal entry");
    assert!(
        matches!(
            refusal,
            RelationalStoreHomeOpenRefusal::Journal(RelationalStoreJournalRefusal::ArtifactFrame {
                position: 6,
                ..
            })
        ),
        "{refusal:?}"
    );
}
