#[allow(unused_imports)] use crate::physical_media_journeys::*;
use {std::path::Path};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::AdmissionError, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::MediaShutdownOutcome, worth_store::physical_runtime::ObservationError, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore};
use {worth_store_physical_backend::FilesystemAccessPosture};









use {crate::physical_media_journeys::process_contender::run_contender, crate::physical_media_journeys::process_contender::spawn_lease_holder};

#[test]
fn media_progression_preserves_identity_and_stales_the_c3_observer() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let c3 = admit_runtime(&root);
    let runtime_identity = c3.runtime_identity();
    let old_observer = c3.observe();
    let media = match c3.try_admit_filesystem_media(media_admission()).into_raw() {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("media progression must succeed"),
    };
    assert_eq!(media.runtime_identity(), runtime_identity);
    assert!(matches!(
        old_observer.snapshot(),
        Err(worth_store::physical_runtime::ObservationError::Stale { .. })
    ));

    let media_observer = media.observer();
    let observation = media_observer.snapshot().unwrap();
    assert_eq!(observation.runtime_identity(), runtime_identity);
    assert_eq!(observation.store_identity(), media.store_identity());
    assert!(matches!(
        worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(&root).unwrap()),
        Err(worth_store::physical_runtime::AdmissionError::DeclaredRootAlreadyAdmitted(_))
    ));

    let stable_identity = media.store_identity();
    assert!(matches!(media.close(), worth_store::physical_runtime::MediaShutdownOutcome::Released(_)));
    let closed_media = media_observer.media_counters();
    assert_eq!(closed_media.ownership_releases(), 1);
    assert_eq!(closed_media.live_file_handles(), 0);
    assert_eq!(closed_media.live_directory_handles(), 0);
    assert_eq!(
        closed_media.directory_opens(),
        closed_media.directory_closes()
    );
    assert!(matches!(
        media_observer.snapshot(),
        Err(worth_store::physical_runtime::ObservationError::Closed { .. })
    ));

    let successor = admit_runtime(&root);
    assert_ne!(successor.runtime_identity(), runtime_identity);
    let successor = match successor
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("successor media progression must succeed"),
    };
    assert_eq!(successor.store_identity(), stable_identity);
    assert_ne!(
        successor.observer().snapshot().unwrap().mutation_owner(),
        observation.mutation_owner()
    );
    assert!(matches!(
        successor.abort(),
        worth_store::physical_runtime::MediaShutdownOutcome::Released(_)
    ));
}

#[test]
fn definite_pre_effect_denial_returns_the_exact_c3_authority() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let runtime = admit_runtime(&root);
    let identity = runtime.runtime_identity();
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::UnmanagedWritersPossible);
    let denial = match runtime.try_admit_filesystem_media(admission).into_raw() {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        _ => panic!("unmanaged access must deny before media effects"),
    };
    assert!(!root.exists());
    let runtime = denial.into_runtime();
    assert_eq!(runtime.runtime_identity(), identity);
    runtime.close();
}

#[test]
fn unexpected_media_drop_releases_both_owners_without_reporting_close() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = match admit_runtime(&root)
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("media progression must succeed"),
    };
    let media_observer = media.observer();
    let before = media_observer.runtime_counters();
    drop(media);
    let after = media_observer.runtime_counters();
    assert_eq!(after.unexpected_drops(), before.unexpected_drops() + 1);
    assert_eq!(after.panic_terminations(), before.panic_terminations());
    let released = media_observer.media_counters();
    assert_eq!(released.ownership_releases(), 1);
    assert_eq!(released.live_file_handles(), 0);
    assert_eq!(released.live_directory_handles(), 0);
    admit_runtime(&root).abort();
}

#[test]
fn panic_unwind_releases_media_authority_without_reporting_close() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = match admit_runtime(&root)
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("media progression must succeed"),
    };
    let media_observer = media.observer();
    let before = media_observer.runtime_counters();
    let unwind = std::panic::catch_unwind(move || {
        let _held_until_unwind = media;
        panic!("controlled unwind");
    });
    assert!(unwind.is_err());
    let after = media_observer.runtime_counters();
    assert_eq!(after.panic_terminations(), before.panic_terminations() + 1);
    assert_eq!(after.unexpected_drops(), before.unexpected_drops());
    let released = media_observer.media_counters();
    assert_eq!(released.ownership_releases(), 1);
    assert_eq!(released.live_file_handles(), 0);
    assert_eq!(released.live_directory_handles(), 0);
    admit_runtime(&root).abort();
}

#[test]
fn independent_observer_reads_closed_media_without_runtime_authority() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = match admit_runtime(&root)
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("media progression must succeed"),
    };
    let expected = media.store_identity();
    assert!(matches!(media.close(), worth_store::physical_runtime::MediaShutdownOutcome::Released(_)));

    let output = std::process::Command::new(worth_store_testkit_executables::physical_media_os_observer())
        .arg(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let observed = String::from_utf8(output.stdout).unwrap();
    let expected = expected
        .bytes()
        .into_iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(observed.trim(), expected);
}

#[test]
fn independent_observer_rejects_every_identity_grammar_corruption_class() {
    type IdentityCorruptionCase = (fn(&mut Vec<u8>), bool);

    use {sha2::Digest, sha2::Sha256};
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = match admit_runtime(&root)
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("media progression must succeed"),
    };
    assert!(matches!(media.close(), worth_store::physical_runtime::MediaShutdownOutcome::Released(_)));
    let identity_path = root.join("namespace/identity");
    let valid = std::fs::read(&identity_path).unwrap();
    let corruptions: [IdentityCorruptionCase; 9] = [
        (|bytes| bytes[0] ^= 1, true),
        (|bytes| bytes[8] = 2, true),
        (|bytes| bytes[12] = 0, true),
        (|bytes| bytes[16] = 2, true),
        (|bytes| bytes[18] = 1, true),
        (|bytes| bytes[20] = 2, true),
        (|bytes| bytes[22] = 15, true),
        (|bytes| bytes[24..40].fill(0), true),
        (|bytes| bytes[40] ^= 1, false),
    ];
    for (corrupt, resign) in corruptions {
        let mut bytes = valid.clone();
        corrupt(&mut bytes);
        if resign {
            let digest = Sha256::digest(&bytes[..40]);
            bytes[40..].copy_from_slice(&digest);
        }
        std::fs::write(&identity_path, bytes).unwrap();
        let output = std::process::Command::new(worth_store_testkit_executables::physical_media_os_observer())
            .arg(&root)
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
    std::fs::write(identity_path, valid).unwrap();
}

#[test]
fn lease_release_is_linearized_against_a_second_process() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let faults = admission.fault_schedule_authority();
    let gate = faults.pause_gate();
    let schedule = faults
        .schedule(Vec::new())
        .unwrap()
        .pause_before_lease_release(gate.clone());
    let admission = admission.with_fault_schedule(schedule);
    let media = match admit_runtime(&root)
        .try_admit_filesystem_media(admission)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("media admission must succeed"),
    };
    let media_observer = media.observer();
    let close = std::thread::spawn(move || media.close());
    gate.wait_until_reached();
    assert!(matches!(
        media_observer.snapshot(),
        Err(worth_store::physical_runtime::ObservationError::Stale { .. })
    ));
    run_contender(&root, "deferred");
    gate.release();
    assert!(matches!(
        close.join().unwrap(),
        worth_store::physical_runtime::MediaShutdownOutcome::Released(_)
    ));
    assert_eq!(media_observer.media_counters().ownership_releases(), 1);
    run_contender(&root, "success");
}

#[test]
fn process_death_releases_the_os_owner_without_a_close_claim() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    run_contender(&root, "die");
    run_contender(&root, "success");
}

#[test]
fn store_counters_lower_only_after_real_execution() {
    use {worth_store::physical_runtime::certification::lower_media_operation_summary};

    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let media = match admit_runtime(&root)
        .try_admit_filesystem_media(media_admission())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("media admission must succeed"),
    };
    let counters = media.media_counters();
    assert!(counters.attempted_operations() > 0);
    let summary = media.certification_operation_summary().unwrap();
    let receipt = lower_media_operation_summary(summary).unwrap();
    assert_eq!(
        receipt.counter_rows().len(),
        37 + worth_store_physical_backend::MediaOperationRole::ALL.len() * 7
    );
    assert_foundational_rows_fail_closed(&receipt);
    assert!(matches!(media.close(), worth_store::physical_runtime::MediaShutdownOutcome::Released(_)));
}
#[path = "child_dispatch/mod.rs"]
pub(crate) mod child_dispatch;
#[path = "hard_link_confinement/mod.rs"]
pub(crate) mod hard_link_confinement;
#[path = "mutation_contention/mod.rs"]
pub(crate) mod mutation_contention;
#[path = "namespace_discovery/mod.rs"]
pub(crate) mod namespace_discovery;
#[path = "partial_effects/mod.rs"]
pub(crate) mod partial_effects;
#[path = "process_contender/mod.rs"]
pub(crate) mod process_contender;
#[path = "transition_outcomes/mod.rs"]
pub(crate) mod transition_outcomes;
