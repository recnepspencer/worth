//! Open refusals: a home that is not this runtime's, a location holding
//! something other than a Store, and a home whose checkpoint or journal is
//! damaged all refuse before service, with a typed refusal, and none of them
//! is treated as an empty home.

#[allow(unused_imports)] use crate::open_refusals::*;
use {std::fs};
use {std::path::Path, std::path::PathBuf};

use {worth_execution::ExecutionRequest};
use {worth_relational::facade::runtime::RelationalRuntimeApi};
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::AnchoredRecordName, worth_store::physical_runtime::AnchoredRecordPublication, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::OrderedJournalPublication, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy};
use {worth_store_relational::RelationalStoreHome, worth_store_relational::RelationalStoreHomeOpenRefusal, worth_store_relational::RelationalStoreHomeRequest, worth_store_relational::RelationalStoreJournalRefusal};

use {crate::durable_run::apply, crate::durable_run::assert_holds, crate::durable_run::open, crate::durable_run::steps_before};
use {crate::schema_transition};
use {crate::selected_checkpoint};
use {crate::workload, crate::workload::Script};

/// A home's journal medium is owned by the one runtime the open home serves
/// (D1): a second runtime opening the same location while it is open is
/// refused, and the first keeps serving.
#[test]
fn a_second_runtime_cannot_open_a_home_that_is_open() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (_directory, location, script, acknowledged) = home_with(2, &execution);
    let home = open(&location, &execution).expect("the home opens");
    let second = open(&location, &execution);
    assert!(
        second.is_err(),
        "a second runtime opened the medium of a home that is open"
    );
    drop(second);
    assert_holds(&home, &script, &acknowledged);
    home.close();
}

#[test]
fn a_home_opened_as_another_runtime_refuses_as_foreign() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (_directory, location, script, acknowledged) = home_with(2, &execution);
    let foreign = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_setup(|runtime| {
            runtime.runtime_name("another-runtime");
        })
        .schema_setup(|setup| {
            setup.schema_registry(crate::schema_transition::registry(crate::schema_transition::SOURCE));
        });
    let refused = worth_store_relational::RelationalStoreHome::open(
        worth_store_relational::RelationalStoreHomeRequest::new(&location, foreign),
        &execution,
    );
    assert!(
        matches!(
            refused,
            Err(worth_store_relational::RelationalStoreHomeOpenRefusal::ForeignHome { .. })
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
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    std::fs::create_dir_all(&location).unwrap();
    std::fs::write(location.join("notes.txt"), b"not a store").unwrap();
    let refused = open(&location, &execution);
    assert!(
        matches!(refused, Err(worth_store_relational::RelationalStoreHomeOpenRefusal::Physical(_))),
        "{:?}",
        refused.map(|_| ())
    );
    let entries: Vec<_> = std::fs::read_dir(&location)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(
        entries,
        ["notes.txt"],
        "the refusal wrote into the location"
    );
    assert_eq!(
        std::fs::read(location.join("notes.txt")).unwrap(),
        b"not a store"
    );
}

#[test]
fn a_damaged_checkpoint_in_the_store_files_refuses_before_service() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
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
            worth_store_relational::RelationalStoreHomeOpenRefusal::Checkpoint(_)
                | worth_store_relational::RelationalStoreHomeOpenRefusal::Physical(_)
        ),
        "{refusal:?}"
    );
}

#[test]
fn a_damaged_journal_entry_in_the_store_files_refuses_before_service() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
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
            worth_store_relational::RelationalStoreHomeOpenRefusal::Journal(_)
                | worth_store_relational::RelationalStoreHomeOpenRefusal::Physical(_)
        ),
        "{refusal:?}"
    );
}

#[test]
fn a_checkpoint_record_store_accepts_but_relational_cannot_readmit_refuses() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (_directory, location, _, _) = home_with(4, &execution);
    let home = open(&location, &execution).expect("the home opens");
    let generation = crate::selected_checkpoint::generation(&home);
    let records = home.certification_serving().anchored_records().unwrap();
    let name = worth_store::physical_runtime::AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR).unwrap();
    let mut payload = records
        .read(name, 1 << 30)
        .unwrap()
        .unwrap()
        .into_bytes()
        .to_vec();
    // Keep the v2 home header (magic, version, P and T), and damage the
    // native checkpoint's format header so readmission must reject it.
    const HOME_HEADER_BYTES: usize = 8 + 2 + 8 + 8;
    payload[HOME_HEADER_BYTES] ^= 0x5a;
    let placement = home_placement();
    records
        .publish(worth_store::physical_runtime::AnchoredRecordPublication::new(
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
        matches!(refusal, worth_store_relational::RelationalStoreHomeOpenRefusal::Readmission(_)),
        "{refusal:?}"
    );
}

#[test]
fn a_journal_entry_store_accepts_but_relational_cannot_readmit_refuses() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (_directory, location, _, acknowledged) = home_with(5, &execution);
    let home = open(&location, &execution).expect("the home opens");
    let mut bytes = acknowledged[4].clone();
    bytes[ARTIFACT_MAGIC.len() + 600] ^= 0x5a;
    home.certification_serving()
        .canonical_commits()
        .unwrap()
        .publish(worth_store::physical_runtime::OrderedJournalPublication::new(
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
            worth_store_relational::RelationalStoreHomeOpenRefusal::Journal(worth_store_relational::RelationalStoreJournalRefusal::ArtifactFrame {
                position: 6,
                ..
            })
        ),
        "{refusal:?}"
    );
}
