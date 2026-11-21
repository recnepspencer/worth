//! Open refusals: a location holding something other than a Store, and a
//! home whose checkpoint or journal is damaged, refuse before service with a
//! typed refusal; none is treated as an empty home. Foreign and contended
//! opens are rows of `incarnation::refusals`.
use worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused as OpenRefusal;

#[allow(unused_imports)]
use crate::open_refusals::*;

use crate::durable_run::open;

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
        matches!(
            refused,
            Err(OpenRefusal::BeforeReadmission(
                worth_store_application::ApplicationHomeOpenRefusal::Physical(_)
            ))
        ),
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
            refusal.source_cause(),
            Some(
                worth_store_application::ApplicationHomeOpenRefusal::Checkpoint(_)
                    | worth_store_application::ApplicationHomeOpenRefusal::Physical(_)
            )
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
            refusal.source_cause(),
            Some(
                worth_store_application::ApplicationHomeOpenRefusal::Relational(
                    worth_store_relational::RelationalSectionReadmissionDenial::Journal(_)
                ) | worth_store_application::ApplicationHomeOpenRefusal::Physical(_)
            )
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
    let records = home.physical_serving_for_test().anchored_records().unwrap();
    let name =
        worth_store::physical_runtime::AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR)
            .unwrap();
    let mut payload = records
        .read(name, 1 << 30)
        .unwrap()
        .unwrap()
        .into_bytes()
        .to_vec();
    // Keep the application manifest and unchanged native-section header;
    // damage the actual native checkpoint so its owner must reject it.
    const HOME_HEADER_BYTES: usize = 24 + 8 + 2 + 8 + 8;
    payload[HOME_HEADER_BYTES] ^= 0x5a;
    let placement = home_placement();
    records
        .publish(
            worth_store::physical_runtime::AnchoredRecordPublication::new(
                name,
                generation,
                payload,
                placement,
                deadline(),
            ),
        )
        .expect("Store accepts the record");
    home.abort_for_test();
    let refusal = assert_refuses_before_service(&location, "unreadmittable checkpoint");
    assert!(
        matches!(
            refusal.source_cause(),
            Some(
                worth_store_application::ApplicationHomeOpenRefusal::Relational(
                    worth_store_relational::RelationalSectionReadmissionDenial::Native(_)
                )
            )
        ),
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
    home.physical_serving_for_test()
        .canonical_commits()
        .unwrap()
        .publish(
            worth_store::physical_runtime::OrderedJournalPublication::new(
                6,
                bytes,
                home_placement(),
                deadline(),
            ),
        )
        .expect("Store accepts the entry");
    home.abort_for_test();
    let refusal = assert_refuses_before_service(&location, "unreadmittable journal entry");
    assert!(
        matches!(
            refusal.source_cause(),
            Some(
                worth_store_application::ApplicationHomeOpenRefusal::Relational(
                    worth_store_relational::RelationalSectionReadmissionDenial::Journal(
                        worth_store_relational::RelationalStoreJournalRefusal::ArtifactFrame {
                            position: 6,
                            ..
                        }
                    )
                )
            )
        ),
        "{refusal:?}"
    );
}
