#[path = "physical_store_offline_observer/c5_c7_observation/bounded_residency_verification.rs"]
mod bounded_residency_verification;
#[path = "physical_store_offline_observer/c5_c7_observation/current_manifest.rs"]
mod current_manifest;
#[path = "physical_store_offline_observer/c5_c7_observation/hostile_physical_truth.rs"]
mod hostile_physical_truth;
#[path = "physical_store_offline_observer/recovery_report.rs"]
mod recovery_report;

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let Some(command) = arguments.next() else {
        usage();
    };
    match command.to_string_lossy().as_ref() {
        "c8-recovery-observe" => run_recovery_observation(arguments),
        "c5-current-manifest" => run_current_manifest(arguments),
        "hostile-physical-truth" => run_hostile_truth(arguments),
        "bounded-residency-verify" => run_bounded_residency(arguments),
        _ => usage(),
    }
}

fn run_recovery_observation(arguments: impl Iterator<Item = std::ffi::OsString>) {
    if let Err(error) = recovery_report::run(arguments) {
        eprintln!("physical_store_offline_observer: {error}");
        std::process::exit(1);
    }
}

fn run_current_manifest(mut arguments: impl Iterator<Item = std::ffi::OsString>) {
    let Some(root) = arguments.next() else {
        usage()
    };
    if arguments.next().is_some() {
        usage()
    }
    current_manifest::run(std::path::Path::new(&root));
}

fn run_hostile_truth(mut arguments: impl Iterator<Item = std::ffi::OsString>) {
    let Some(root) = arguments.next() else {
        usage()
    };
    if arguments.next().is_some() {
        usage()
    }
    hostile_physical_truth::run(std::path::Path::new(&root));
}

fn run_bounded_residency(mut arguments: impl Iterator<Item = std::ffi::OsString>) {
    let Some(root) = arguments.next() else {
        usage()
    };
    let Some(configuration) = arguments.next() else {
        usage()
    };
    if arguments.next().is_some() {
        usage()
    }
    bounded_residency_verification::run(
        std::path::Path::new(&root),
        std::path::Path::new(&configuration),
    );
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn usage() -> ! {
    eprintln!(
        "usage: physical_store_offline_observer c8-recovery-observe \
         <store-root> <report-output> <maximum-directory-entries> \
         <maximum-directories> <maximum-artifacts> <maximum-bytes> | \
         c5-current-manifest <store-root> | hostile-physical-truth <store-root> | \
         bounded-residency-verify <store-root> <configuration>"
    );
    std::process::exit(2);
}
