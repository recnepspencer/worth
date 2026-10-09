use std::num::{NonZeroU16, NonZeroU64};
use std::path::Path;
use std::process::Command;

use worth_foundational::{
    aspects, AspectValue, ContractValidationInput, InternedString, ScalarAspectType,
};
use worth_proof::{AdmittedBlobReleaseProof, TransitionOutcome};
use worth_store::aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StoreAspectIdentity,
    StorePhysicalAuthorityWitness, StorePhysicalBoundaryWitness,
    ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};
use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestFailure,
    BlobReadLimits, BlobReclaimDisposition, BlobReclaimLimits, BlobReclaimRequest,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline,
};
use worth_store_authority::require_current_store_authority;
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_security::{
    admit_store_security_scope, StoreAuthenticityRequirement, StoreAuthenticityRequirementClass,
    StoreCustodyPosture, StoreKeyScope, StoreKeyVersionPosture,
    StoreSecurityScopeAdmissionExpectation, StoreSecurityScopeAdmissionRequest, StoreTenantScope,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

#[path = "production_entry/release_reopen.rs"]
mod release_reopen;

#[path = "production_entry/blob_scope_admission.rs"]
mod blob_scope_admission;
use blob_scope_admission::admitted_blob_scope;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/certified_release_serving.rs"]
mod certified_release_serving;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/pending_wal_world.rs"]
mod pending_wal_world;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/pending_wal_fold.rs"]
mod pending_wal_fold;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/selected_batch_multi_pending.rs"]
mod selected_batch_multi_pending;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/pending_successor_above_history.rs"]
mod pending_successor_above_history;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/published_above_checkpoint.rs"]
mod published_above_checkpoint;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/three_batch_fold.rs"]
mod three_batch_fold;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/selected_tag7_pruned_wal.rs"]
mod selected_tag7_pruned_wal;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/accumulator_only_pending_wal.rs"]
mod accumulator_only_pending_wal;

#[path = "production_entry/historical_release_only.rs"]
mod historical_release_only;
#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/mixed_retained_control.rs"]
mod mixed_retained_control;
#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/retired_tail_append.rs"]
mod retired_tail_append;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/maintenance_result.rs"]
mod maintenance_result;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/production_custody_serving.rs"]
mod production_custody_serving;

#[cfg(feature = "certification-test-authority")]
#[test]
fn selected_custody_rejects_checkpoint_substitution_after_seal_before_serving_open() {
    certified_release_serving::run(certified_release_serving::Mutation::AfterSealCheckpoint);
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn selected_custody_rejects_control_substitution_after_seal_before_serving_open() {
    certified_release_serving::run(certified_release_serving::Mutation::AfterSealControl);
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn selected_custody_rejects_wal_substitution_after_seal_before_serving_open() {
    certified_release_serving::run(certified_release_serving::Mutation::AfterSealWal);
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn selected_custody_rejects_route_substitution_after_seal_before_serving_open() {
    certified_release_serving::run(certified_release_serving::Mutation::AfterSealRoute);
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn selected_tier_and_released_custody_open_serving_from_one_composite_seal() {
    certified_release_serving::run_tier_and_release();
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn selected_empty_no_release_custody_opens_serving_and_checkpoints() {
    certified_release_serving::run_empty_no_release();
}

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/tier_epoch_reopen.rs"]
mod tier_epoch_reopen;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/tier_release_pending_wal.rs"]
mod tier_release_pending_wal;

#[cfg(feature = "certification-test-authority")]
#[path = "production_entry/no_release_failed_ingest.rs"]
mod no_release_failed_ingest;

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
    let output = release_reopen::run(1, false);
    assert!(
        output.status.success(),
        "fresh release recovery: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn terminal_release_survives_checkpoint_and_fresh_recovery() {
    let output = release_reopen::run(1024, true);
    assert!(
        output.status.success(),
        "terminal release recovery: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn two_releases_recover_with_current_batch_history() {
    release_reopen::two_batch::run(false);
}

#[test]
fn two_releases_recover_after_accumulator_only_carryforward() {
    release_reopen::two_batch::run(true);
}

#[test]
fn two_distinct_released_generations_recover_with_current_batch() {
    release_reopen::two_generations::run(false);
}

#[test]
fn two_distinct_released_generations_recover_after_carryforward() {
    release_reopen::two_generations::run(true);
}

#[test]
fn failed_ingest_then_independent_release_recovers_from_selected_controls() {
    release_reopen::mixed_failed_ingest::run();
}

#[test]
fn release_candidate_certificate_crash_keeps_prior_selected_custody() {
    release_reopen::candidate_crash::run();
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn release_candidate_crash_recovery_allows_successor_checkpoint() {
    release_reopen::candidate_crash::run_checkpoint_liveness();
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn genuine_release_custody_rejoins_store_media_before_serving() {
    certified_release_serving::run(certified_release_serving::Mutation::None);
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn changed_selected_checkpoint_after_c8_claim_denies_store_rejoin() {
    certified_release_serving::run(certified_release_serving::Mutation::AfterClaimCheckpoint);
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn changed_selected_checkpoint_before_final_store_reread_denies_seal() {
    certified_release_serving::run(
        certified_release_serving::Mutation::BeforeFinalRejoinCheckpoint,
    );
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn changed_selected_control_before_final_store_reread_denies_seal() {
    certified_release_serving::run(certified_release_serving::Mutation::BeforeFinalRejoinControl);
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
    let output = Command::new(env!("CARGO_BIN_EXE_physical_store_recover"))
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

fn run_entry(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_physical_store_recover"))
        .arg(root)
        .arg("--bounded-profile=c8-phase2-admission-v1")
        .output()
        .expect("run production recovery entry")
}

fn runtime_text(stderr: &str) -> &str {
    stderr
        .split_once(" runtime ")
        .expect("runtime prefix")
        .1
        .split_once(" at root generation")
        .expect("runtime suffix")
        .0
}

fn initialized_recovery_world(label: &str) -> PhysicalResidencyStoreWorld {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x41; 32], b"production-entry-redo");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x42; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("production entry checkpoint admission must succeed")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    world
}

fn test_limits() -> worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration {
        selector_candidates: 1,
        checkpoint_candidates: 1,
        manifest_bytes: 1,
        manifest_entries: 1,
        wal_segments: 1,
        wal_frames: 1,
        wal_bytes: 1,
        redo_targets: 1,
        redo_bytes: 1,
        distinct_pages_and_extents: 1,
        operation_bindings: 1,
        staging_bytes: 1,
        recovery_memory_bytes: 1,
        dirty_frames: 1,
        concurrent_commands: 1,
        publication_effects: 1,
        cleanup_candidates: 1,
        cleanup_bytes: 1,
        observation_bytes: 1,
    }
}
