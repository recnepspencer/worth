//! A killed high-index anchor advance must replay from WAL, not its old root.
use super::super::killed_child;
use super::*;
use std::{path::PathBuf, thread, time::Instant};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, PhysicalMutationDeadline,
};

fn publish(
    home: &worth_store_relational::RelationalStoreHome,
    text: &str,
    generation: u64,
    payload: Vec<u8>,
) {
    home.certification_serving()
        .anchored_records()
        .unwrap()
        .publish(AnchoredRecordPublication::new(
            AnchoredRecordName::new(text).unwrap(),
            generation,
            payload,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap();
}

#[test]
fn wal_replays_the_thirty_second_anchor_advance_after_process_death() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let markers = tempfile::tempdir().unwrap();
    assert_eq!(
        killed_child::run_and_kill_observing(
            "incarnation::anchor_replay::high_anchor_child",
            &[
                ("WORTH_HIGH_ANCHOR_LOCATION", &location),
                ("WORTH_HIGH_ANCHOR_MARKERS", markers.path())
            ],
            &markers.path().join("ready"),
            &markers.path().join("finished"),
            || assert_contended(&location)
        ),
        killed_child::ChildEnd::KilledAtPark
    );
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(&location, &execution).unwrap();
    let records = home.certification_serving().anchored_records().unwrap();
    let record = records
        .read(AnchoredRecordName::new("zz.high").unwrap(), 1024)
        .unwrap()
        .unwrap();
    assert_eq!(record.version().generation(), 2);
    assert_eq!(record.bytes(), &[99]);
    for i in 0..30 {
        assert_eq!(
            records
                .read(
                    AnchoredRecordName::new(&format!("anchor.{i:02}")).unwrap(),
                    1024
                )
                .unwrap()
                .unwrap()
                .bytes(),
            &[i as u8]
        );
    }
    home.close();
}

#[test]
fn high_anchor_child() {
    let Some(location) = std::env::var_os("WORTH_HIGH_ANCHOR_LOCATION") else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os("WORTH_HIGH_ANCHOR_MARKERS").unwrap());
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(Path::new(&location), &execution).unwrap();
    for i in 0..30 {
        publish(&home, &format!("anchor.{i:02}"), 0, vec![i as u8]);
    }
    publish(&home, "zz.high", 0, vec![1]);
    let gate = home
        .certification_serving()
        .certification_pause_physical_mutation_at(Seam::AfterWalDurability);
    thread::scope(|scope| {
        let worker = scope.spawn(|| publish(&home, "zz.high", 1, vec![99]));
        let deadline = Instant::now() + killed_child::PARK_DEADLINE;
        while !gate.await_arrival() {
            if worker.is_finished() || Instant::now() >= deadline {
                gate.release();
                panic!("high anchor did not reach WAL durability");
            }
        }
        killed_child::announce(&markers.join("ready"), b"high anchor WAL durable");
        killed_child::park();
    });
}

pub(super) fn placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    use worth_store::physical_runtime::*;
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap()
}
