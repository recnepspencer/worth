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
            match open(&location, build(), initialized.clone()) {
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
            }
            assert_eq!(initialized.load(Ordering::SeqCst), 0);
        },
    );
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark);
    // A third real process attempts readmission after an OS kill, without any
    // close protocol. Its exact Native refusal is evidence, never open success.
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::application::admission::ownership::host_reader_child",
            "--nocapture",
        ])
        .env("WORTH_HOST_LEASE_HOME", &location)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "reader: {stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("HOST_AFTER_KILL="),
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
    let home = open(Path::new(&path), build(), initialized.clone())
        .unwrap_or_else(|cause| panic!("writer: {cause:?}"));
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert!(home.is_directory());
    let ready = std::path::PathBuf::from(std::env::var_os("WORTH_HOST_LEASE_READY").unwrap());
    crate::killed_child::announce(&ready, b"serving:initialized-once");
    // Keep the actual public Host and its continuous Store lease alive.
    std::hint::black_box(&home);
    crate::killed_child::park();
}

#[test]
fn host_reader_child() {
    let Some(path) = std::env::var_os("WORTH_HOST_LEASE_HOME") else {
        return;
    };
    let initialized = Arc::new(AtomicUsize::new(0));
    match open(Path::new(&path), build(), initialized.clone()) {
        Ok(home) => {
            assert!(home.is_directory());
            println!("HOST_AFTER_KILL=Serving");
            drop(home);
        }
        Err(Refused::NativeReadmission { home, .. }) => {
            // P-A owns this step. Require its actual owner refusal, rather
            // than accepting a physical denial or replaying the initializer.
            println!("HOST_AFTER_KILL=NativeReadmission({:?})", home.reason());
            assert_known_native_readmission_block(home.reason());
        }
        Err(cause) => panic!("lease must be available after process death: {cause:?}"),
    }
    assert_eq!(initialized.load(Ordering::SeqCst), 0);
}
