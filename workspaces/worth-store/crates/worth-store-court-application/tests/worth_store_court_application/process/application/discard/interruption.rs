//! Actual child-process death after durable intent and during filesystem removal.
use super::*;
use worth_store::physical_runtime::{
    certification::{MediaFaultDirective, MediaOperationRole as Role},
    FilesystemAccessPosture, FilesystemMediaAdmission,
};

#[test]
fn killed_discard_retains_the_tombstone_and_retry_finishes_each_durable_removal_boundary() {
    for (role, ordinal) in [("intent", 1), ("delete", 1), ("delete", 2)] {
        let directory = tempfile::tempdir().unwrap();
        let location = directory.path().join("home");
        let request = crate::workload::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&request);
        let home = crate::durable_run::open(&location, &execution).unwrap();
        close(home, &execution);
        let selection = std::path::PathBuf::from(format!("{role}-{ordinal}"));
        let ready = directory.path().join("ready");
        let finished = directory.path().join("finished");
        let end = crate::killed_child::run_and_kill_observing(
            "process::application::discard::interruption::discard_child",
            &[
                ("WORTH_DISCARD_LOCATION", &location),
                ("WORTH_DISCARD_SELECTION", &selection),
                ("WORTH_DISCARD_MARKERS", directory.path()),
            ],
            &ready,
            &finished,
            || {
                assert!(location.join("discard.pending").is_file());
                assert_discard_pending(&location);
                assert!(matches!(
                    discard(&location),
                    Err(Denial::MutationOwnerContended)
                ));
            },
        );
        assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark);
        assert_discard_pending(&location);
        discard(&location).unwrap();
        let home = crate::durable_run::open(&location, &execution).unwrap();
        assert_eq!(home.incarnation().get(), 1);
        close(home, &execution);
    }
}

#[test]
fn discard_child() {
    let Some(location) = std::env::var_os("WORTH_DISCARD_LOCATION") else {
        return;
    };
    let selection = std::env::var("WORTH_DISCARD_SELECTION").unwrap();
    let markers = std::path::PathBuf::from(std::env::var_os("WORTH_DISCARD_MARKERS").unwrap());
    let (role, ordinal) = match selection.as_str() {
        "intent-1" => (Role::SynchronizeDirectoryPublication, 2),
        "delete-1" => (Role::Delete, 1),
        "delete-2" => (Role::Delete, 2),
        _ => panic!("unknown child discard boundary"),
    };
    let media =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = media.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            role,
            ordinal,
            MediaFaultDirective::PauseAfter(gate.clone()),
        )])
        .unwrap();
    std::thread::scope(|threads| {
        threads.spawn(move || {
            worth_store_application::ApplicationHomeDiscard::new(location.into())
                .with_fault_schedule(schedule)
                .execute()
                .unwrap();
        });
        gate.wait_until_reached();
        crate::killed_child::announce(&markers.join("ready"), b"discard held");
        crate::killed_child::park();
    });
}
