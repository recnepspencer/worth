use super::*;
use worth_store_application::{ApplicationHomeOpenRefusal, ApplicationHomePhysicalRefusalKind};

#[test]
fn competing_host_processes_admit_exactly_one_writer_until_process_death() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let ready = directory.path().join("ready");
    let finished = directory.path().join("finished");
    let end = crate::killed_child::run_and_kill_observing(
        "process::application::admission::ownership::host_writer_child",
        &[
            ("WORTH_HOST_LEASE_HOME", &location),
            ("WORTH_HOST_LEASE_READY", &ready),
        ],
        &ready,
        &finished,
        || {
            assert_eq!(std::fs::read(&ready).unwrap(), b"serving:initialized-once");
            let initialized = Arc::new(AtomicUsize::new(0));
            with_open(
                &location,
                build(),
                initialized.clone(),
                |opened, _contacts| match opened {
                    Err(Refused::Physical {
                        cause: ApplicationHomeOpenRefusal::Physical(cause),
                        ..
                    }) => {
                        assert_eq!(
                            cause.kind(),
                            ApplicationHomePhysicalRefusalKind::MutationOwnerContended
                        );
                    }
                    Err(cause) => panic!("competing Host must report lease contention: {cause:?}"),
                    Ok(_) => panic!("two processes cannot both serve one Home"),
                },
            );
            assert_eq!(initialized.load(Ordering::SeqCst), 0);
        },
    );
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark);
    // A third real process attempts readmission after an OS kill, without any
    // close protocol. Only actual Serving can satisfy this positive reopen.
    let mut reader = std::process::Command::new(std::env::current_exe().unwrap());
    reader
        .args([
            "--exact",
            "process::application::admission::ownership::host_reader_child",
            "--nocapture",
        ])
        .env("WORTH_HOST_LEASE_HOME", &location);
    let output =
        worth_query_testkit_host::facade::host::process_deadline::output(&mut reader).unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "reader: {stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("HOST_AFTER_KILL=Serving"),
        "reader observation absent: {stdout}"
    );
    println!("{stdout}");
}

#[test]
fn host_writer_child() {
    let Some(path) = std::env::var_os("WORTH_HOST_LEASE_HOME") else {
        return;
    };
    let initialized = Arc::new(AtomicUsize::new(0));
    with_open(
        Path::new(&path),
        build(),
        initialized.clone(),
        |opened, _contacts| {
            let home = opened
                .as_ref()
                .unwrap_or_else(|cause| panic!("writer: {cause:?}"));
            assert_eq!(initialized.load(Ordering::SeqCst), 1);
            assert!(home.is_directory());
            let ready =
                std::path::PathBuf::from(std::env::var_os("WORTH_HOST_LEASE_READY").unwrap());
            crate::killed_child::announce(&ready, b"serving:initialized-once");
            // Keep the actual public Host and its continuous Store lease alive.
            std::hint::black_box(home);
            crate::killed_child::park();
        },
    );
}

#[test]
fn host_reader_child() {
    let Some(path) = std::env::var_os("WORTH_HOST_LEASE_HOME") else {
        return;
    };
    let initialized = Arc::new(AtomicUsize::new(0));
    with_open(
        Path::new(&path),
        build(),
        initialized.clone(),
        |opened, _contacts| match opened {
            Ok(home) => {
                assert!(home.is_directory());
                println!("HOST_AFTER_KILL=Serving");
            }
            Err(cause) => panic!("lease must be available after process death: {cause:?}"),
        },
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 0);
}
