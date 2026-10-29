#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::*;
use {std::num::NonZeroU16, std::num::NonZeroU64};
use {std::path::Path};
use {std::process::Command};

use {worth_foundational::aspects, worth_foundational::AspectValue, worth_foundational::ContractValidationInput, worth_foundational::InternedString, worth_foundational::ScalarAspectType};
use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
use {worth_store::aspect_native::StoreAspectAuthorityInput, worth_store::aspect_native::StoreAspectBoundaryFact, worth_store::aspect_native::StoreAspectIdentity, worth_store::aspect_native::StorePhysicalAuthorityWitness, worth_store::aspect_native::StorePhysicalBoundaryWitness, worth_store::aspect_native::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE};
use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline};
use {worth_store_authority::require_current_store_authority};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_security::admit_store_security_scope, worth_store_security::StoreAuthenticityRequirement, worth_store_security::StoreAuthenticityRequirementClass, worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope, worth_store_security::StoreKeyVersionPosture, worth_store_security::StoreSecurityScopeAdmissionExpectation, worth_store_security::StoreSecurityScopeAdmissionRequest, worth_store_security::StoreTenantScope};
use {worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};




use {worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope};














#[cfg(all())]
use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle};
















#[cfg(all())]
#[test]
fn selected_custody_rejects_checkpoint_substitution_after_seal_before_serving_open() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::AfterSealCheckpoint);
}

#[cfg(all())]
#[test]
fn selected_custody_rejects_control_substitution_after_seal_before_serving_open() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::AfterSealControl);
}

#[cfg(all())]
#[test]
fn selected_custody_rejects_wal_substitution_after_seal_before_serving_open() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::AfterSealWal);
}

#[cfg(all())]
#[test]
fn selected_custody_rejects_route_substitution_after_seal_before_serving_open() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::AfterSealRoute);
}

#[cfg(all())]
#[test]
fn selected_tier_and_released_custody_open_serving_from_one_composite_seal() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run_tier_and_release();
}

#[cfg(all())]
#[test]
fn selected_empty_no_release_custody_opens_serving_and_checkpoints() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::empty_no_release::run_empty_no_release();
}







#[test]
fn production_entry_admits_an_existing_store_in_a_fresh_process() {
    let world = initialized_recovery_world("production-entry");
    let retained_root = world.retained_root();
    let root = retained_root.path().to_path_buf();
    let store_identity = world.serving().store_identity();
    drop(world);

    let first = run_entry(&root);
    let second = run_entry(&root);

    assert!(
        first.status.success(),
        "production entry failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(second.status.success());
    let first_stderr = String::from_utf8_lossy(&first.stderr);
    let second_stderr = String::from_utf8_lossy(&second.stderr);
    assert!(first_stderr.contains("recovered Store"));
    assert!(first_stderr.contains(&format!("{:?}", store_identity.bytes())));
    assert_ne!(runtime_text(&first_stderr), runtime_text(&second_stderr));

    let authority = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire(
        root,
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current(),
        worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(test_limits())
            .expect("test limits"),
    )
    .expect("production entry released its root lease");
    drop(authority);
}

#[test]
fn production_entry_refuses_absent_incomplete_and_contended_roots() {
    let parent = tempfile::tempdir().expect("test root parent");
    let absent = parent.path().join("absent");
    let absent_output = run_entry(&absent);
    assert!(!absent_output.status.success());
    assert!(!absent.exists());

    let incomplete = parent.path().join("incomplete");
    std::fs::create_dir(&incomplete).expect("incomplete root");
    let incomplete_output = run_entry(&incomplete);
    assert!(!incomplete_output.status.success());
    assert_eq!(
        std::fs::read_dir(&incomplete)
            .expect("unchanged root")
            .count(),
        0
    );

    let world = initialized_recovery_world("production-contention");
    let retained_root = world.retained_root();
    let root = retained_root.path().to_path_buf();
    let serving_contended = run_entry(&root);
    assert!(!serving_contended.status.success());
    assert!(String::from_utf8_lossy(&serving_contended.stderr).contains("OwnershipContended"));
    drop(world);
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(test_limits())
        .expect("test limits");
    let owner = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire(
        root.clone(),
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current(),
        limits,
    )
    .expect("parent owns recovery root");
    let contended = run_entry(&root);
    assert!(!contended.status.success());
    assert!(String::from_utf8_lossy(&contended.stderr).contains("OwnershipContended"));
    drop(owner);
}

#[test]
fn completed_release_first_batch_survives_checkpoint_and_fresh_recovery() {
    let output = worth_store_testkit_recovery_runtime::production_entry::release_reopen::run(1, false);
    assert!(
        output.status.success(),
        "fresh release recovery: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn terminal_release_survives_checkpoint_and_fresh_recovery() {
    let output = worth_store_testkit_recovery_runtime::production_entry::release_reopen::run(1024, true);
    assert!(
        output.status.success(),
        "terminal release recovery: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn two_releases_recover_with_current_batch_history() {
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_batch::run(false);
}

#[test]
fn two_releases_recover_after_accumulator_only_carryforward() {
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_batch::run(true);
}

#[test]
fn two_distinct_released_generations_recover_with_current_batch() {
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_generations::run(false);
}

#[test]
fn two_distinct_released_generations_recover_after_carryforward() {
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::two_generations::run(true);
}

#[test]
fn failed_ingest_then_independent_release_recovers_from_selected_controls() {
    crate::production_entry::release_reopen::mixed_failed_ingest::run();
}

#[test]
fn release_candidate_certificate_crash_keeps_prior_selected_custody() {
    worth_store_testkit_recovery_runtime::production_entry::release_reopen::candidate_crash::run();
}

#[cfg(all())]
#[test]
fn release_candidate_crash_recovery_allows_successor_checkpoint() {
    worth_store_court_recovery_runtime_replication_production_entry_certified_release_serving::process::recovery_runtime::production_entry::release_reopen::candidate_crash::run_checkpoint_liveness();
}

#[cfg(all())]
#[test]
fn genuine_release_custody_rejoins_store_media_before_serving() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::None);
}

#[cfg(all())]
#[test]
fn changed_selected_checkpoint_after_c8_claim_denies_store_rejoin() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::AfterClaimCheckpoint);
}

#[cfg(all())]
#[test]
fn changed_selected_checkpoint_before_final_store_reread_denies_seal() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::BeforeFinalRejoinCheckpoint,
    );
}

#[cfg(all())]
#[test]
fn changed_selected_control_before_final_store_reread_denies_seal() {
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::run(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation::BeforeFinalRejoinControl);
}

#[test]
fn production_entry_rejects_a_report_path_inside_the_store_root() {
    let world = initialized_recovery_world("production-report-safety");
    let retained_root = world.retained_root();
    let root = retained_root.path().to_path_buf();
    let before = std::fs::read_dir(&root)
        .expect("read store before report rejection")
        .map(|entry| entry.expect("store entry").file_name())
        .collect::<Vec<_>>();
    drop(world);

    let report = root.join("report-inside-store.bin");
    let output = std::process::Command::new(worth_store_testkit_executables::physical_store_recover())
        .arg(&root)
        .arg("--bounded-profile=c8-phase2-admission-v1")
        .arg(format!("--report={}", report.display()))
        .output()
        .expect("run report safety entry");
    assert!(!output.status.success());
    assert!(!report.exists());
    let after = std::fs::read_dir(&root)
        .expect("read store after report rejection")
        .map(|entry| entry.expect("store entry").file_name())
        .collect::<Vec<_>>();
    assert_eq!(after, before);
}
#[cfg(all())]
#[path = "anchored_records/mod.rs"]
pub(crate) mod anchored_records;
#[cfg(all())]
#[path = "before_first_checkpoint/mod.rs"]
pub(crate) mod before_first_checkpoint;
#[cfg(all())]
#[path = "maintenance_result/mod.rs"]
pub(crate) mod maintenance_result;
#[cfg(all())]
#[path = "mixed_retained_control/mod.rs"]
pub(crate) mod mixed_retained_control;
#[cfg(all())]
#[path = "no_release_failed_ingest/mod.rs"]
pub(crate) mod no_release_failed_ingest;
#[cfg(all())]
#[path = "pending_successor_above_history/mod.rs"]
pub(crate) mod pending_successor_above_history;
#[cfg(all())]
#[path = "pending_wal_fold/mod.rs"]
pub(crate) mod pending_wal_fold;
#[cfg(all())]
#[path = "physical_checkpoint_oracle/mod.rs"]
pub(crate) mod physical_checkpoint_oracle;
#[cfg(all())]
#[path = "production_custody_serving/mod.rs"]
pub(crate) mod production_custody_serving;
#[cfg(all())]
#[path = "retired_tail_append/mod.rs"]
pub(crate) mod retired_tail_append;
#[cfg(all())]
#[path = "selected_batch_multi_pending/mod.rs"]
pub(crate) mod selected_batch_multi_pending;
#[cfg(all())]
#[path = "selected_tag7_pruned_wal/mod.rs"]
pub(crate) mod selected_tag7_pruned_wal;
#[cfg(all())]
#[path = "staging_residue_recovery/mod.rs"]
pub(crate) mod staging_residue_recovery;
#[cfg(all())]
#[path = "three_batch_fold/mod.rs"]
pub(crate) mod three_batch_fold;
#[cfg(all())]
#[path = "tier_epoch_reopen/mod.rs"]
pub(crate) mod tier_epoch_reopen;
#[cfg(all())]
#[path = "tier_release_pending_wal/mod.rs"]
pub(crate) mod tier_release_pending_wal;
