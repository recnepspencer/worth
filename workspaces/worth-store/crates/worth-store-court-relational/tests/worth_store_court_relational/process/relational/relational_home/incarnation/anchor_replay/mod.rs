//! A killed high-index anchor advance must replay from WAL, not its old root.
#[allow(unused_imports)]
use crate::incarnation::anchor_replay::*;
use {crate::durable_run::open, crate::incarnation::assert_contended};

#[test]
fn wal_replays_the_thirty_second_anchor_advance_after_process_death() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let markers = tempfile::tempdir().unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill_observing(
            "process::relational::relational_home::incarnation::anchor_replay::high_anchor_child",
            &[
                ("WORTH_HIGH_ANCHOR_LOCATION", &location),
                ("WORTH_HIGH_ANCHOR_MARKERS", markers.path())
            ],
            &markers.path().join("ready"),
            &markers.path().join("finished"),
            || assert_contended(&location)
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = open(&location, &execution).unwrap();
    let records = home.certification_serving().anchored_records().unwrap();
    let record = records
        .read(
            worth_store::physical_runtime::AnchoredRecordName::new("zz.high").unwrap(),
            1024,
        )
        .unwrap()
        .unwrap();
    assert_eq!(record.version().generation(), 2);
    assert_eq!(record.bytes(), &[99]);
    for i in 0..30 {
        assert_eq!(
            records
                .read(
                    worth_store::physical_runtime::AnchoredRecordName::new(&format!(
                        "anchor.{i:02}"
                    ))
                    .unwrap(),
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
    let markers = std::path::PathBuf::from(std::env::var_os("WORTH_HIGH_ANCHOR_MARKERS").unwrap());
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = open(std::path::Path::new(&location), &execution).unwrap();
    for i in 0..30 {
        publish(&home, &format!("anchor.{i:02}"), 0, vec![i as u8]);
    }
    publish(&home, "zz.high", 0, vec![1]);
    let gate = home
        .certification_serving()
        .certification_pause_physical_mutation_at(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| publish(&home, "zz.high", 1, vec![99]));
        let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
        while !gate.await_arrival() {
            if worker.is_finished() || std::time::Instant::now() >= deadline {
                gate.release();
                panic!("high anchor did not reach WAL durability");
            }
        }
        crate::killed_child::announce(&markers.join("ready"), b"high anchor WAL durable");
        crate::killed_child::park();
    });
}
