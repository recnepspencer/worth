use sha2::Digest;
use tempfile::tempdir;
#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::physical_work::failure::recovery::*;

use worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture;

#[test]
fn crash_after_target_effect_leaves_a_durable_obligation_that_fences_reopen() {
    let root = tempdir().unwrap();
    let writer = child("physical_work_crash_writer", root.path());
    assert!(
        !writer.status.success(),
        "faulting writer must die before recovery-obligation retirement"
    );
    let reopener = child("physical_work_crash_reopener", root.path());
    assert!(
        reopener.status.success(),
        "reopener failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&reopener.stdout),
        String::from_utf8_lossy(&reopener.stderr),
    );
    assert!(String::from_utf8(reopener.stdout)
        .unwrap()
        .lines()
        .any(|line| line == "C5_PHYSICAL_RECOVERY_INSPECTION"));
}

#[test]
fn corrupted_recovery_evidence_is_fenced_without_forging_a_locator() {
    let root = tempdir().unwrap();
    let writer = child("physical_work_crash_writer", root.path());
    assert!(!writer.status.success());
    let journal = root.path().join("families/physical-work");
    let entry = std::fs::read_dir(&journal)
        .unwrap()
        .next()
        .expect("crash must retain one recovery record")
        .unwrap()
        .path();
    let mut bytes = std::fs::read(&entry).unwrap();
    bytes[56] ^= 0x80;
    std::fs::write(&entry, bytes).unwrap();
    let (profile, _, _) = work_fixture();
    let serving = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile(root.path(), profile);

    assert!(serving.physical_recovery_obligations().is_empty());
    assert!(serving.physical_recovery_evidence_damaged());
    assert!(serving.close_plan().execute().requires_inspection());
}

#[test]
fn checksum_valid_noncanonical_recovery_evidence_cannot_forge_a_locator() {
    let root = tempdir().unwrap();
    let writer = child("physical_work_crash_writer", root.path());
    assert!(!writer.status.success());
    let journal = root.path().join("families/physical-work");
    let entry = std::fs::read_dir(&journal)
        .unwrap()
        .next()
        .expect("crash must retain one recovery record")
        .unwrap()
        .path();
    let mut bytes = std::fs::read(&entry).unwrap();
    bytes[10] = 1;
    let checksum = sha2::Sha256::digest(&bytes[..128]);
    bytes[128..].copy_from_slice(&checksum);
    std::fs::write(&entry, bytes).unwrap();
    let (profile, _, _) = work_fixture();
    let serving = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile(root.path(), profile);

    assert!(serving.physical_recovery_obligations().is_empty());
    assert!(serving.physical_recovery_evidence_damaged());
    assert!(serving.close_plan().execute().requires_inspection());
}

#[test]
fn checksum_valid_wrong_pending_scope_is_rejected_before_owner_interpretation() {
    let root = tempdir().unwrap();
    let writer = child("physical_work_crash_writer", root.path());
    assert!(!writer.status.success());
    let journal = root.path().join("families/physical-work");
    let entry = std::fs::read_dir(&journal)
        .unwrap()
        .next()
        .expect("crash must retain one recovery record")
        .unwrap()
        .path();
    let name = entry.file_name().unwrap().to_str().unwrap();
    let mut fields = name.split('-');
    let prefix = fields.next().unwrap();
    let runtime = fields.next().unwrap();
    let generation = fields.next().unwrap();
    let operation = fields.next().unwrap().strip_suffix(".pending").unwrap();
    let other_operation = u64::from_str_radix(operation, 16).unwrap() + 1;
    let wrong_name = format!("{prefix}-{runtime}-{generation}-{other_operation:016x}.pending");
    std::fs::rename(&entry, journal.join(&wrong_name)).unwrap();
    let (profile, _, _) = work_fixture();
    let serving = worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile(root.path(), profile);

    assert!(serving.physical_recovery_obligations().is_empty());
    let observations = serving.physical_recovery_admission_observations();
    assert_eq!(observations.len(), 1);
    assert_eq!(
        observations[0].subject(),
        &worth_store::physical_runtime::PhysicalWorkRecoveryObservationSubject::PendingFile(
            wrong_name.into()
        )
    );
    assert!(matches!(
        observations[0].outcome(),
        worth_store::physical_runtime::PhysicalWorkRecoveryAdmissionOutcome::Rejected {
            rejection: worth_store::physical_runtime::PhysicalWorkRecoveryIngressRejection::Integrity(
                worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
            ),
            ..
        } if localization.cause()
            == worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch
    ));
    let counters = serving.physical_recovery_admission_counters();
    assert_eq!(counters.attempted(), 1);
    assert_eq!(counters.admitted_count(), 0);
    assert_eq!(counters.rejected_before_owner_interpretation_count(), 1);
    assert_eq!(counters.owner_interpretation_entries(), 0);
    assert!(serving.close_plan().execute().requires_inspection());
}
