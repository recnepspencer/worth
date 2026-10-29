//! A genuine process-killed V3 released drop, retained for C8/Store rejoin
//! tests that must share one pending descriptor and its physical controls.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::*;
use {std::cell::Cell, std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};

use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordReadDenial};
use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

use {std::num::NonZeroU64, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving};

use {worth_store_testkit_recovery_runtime::pending_wal_world::descriptor_park::park_at_descriptor_wal};
pub(super) use {};

#[test]
fn pending_wal_world_reaches_verified_c8_and_sealed_serving() {
    let world = first();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("real pending V3 WAL recovery failed: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("C8 pending V3 custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn second_v3_descriptor_after_certified_retirement_checkpoint_is_durable() {
    let world = first();
    world.kill_second_after_certified_retirement();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "third-process C8 blocked at {:?}, effects={}, planning={:?}",
                blocked.cause(),
                blocked.recovery_effects(),
                blocked.evidence().planning_counters
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "third-process C8 publication indeterminate, effects={}, handoff={:?}, reopen={:?}",
                indeterminate.recovery_effects(),
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure()
            ),
            other => panic!("third-process C8 must recover second V3 WAL: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("second V3 selected custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn distinct_v3_descriptor_before_checkpoint_reaches_durable_wal() {
    let world = first();
    world.kill_distinct_release_before_checkpoint();
}

#[test]
fn distinct_v3_descriptor_before_checkpoint_is_durable() {
    let world = first();
    world.kill_distinct_release_before_checkpoint();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "same-checkpoint distinct V3 blocked at {:?}, denial={:?}, artifact={:?}, effects={}",
                blocked.cause(),
                blocked.evidence().planning_denial,
                blocked.evidence().artifact,
                blocked.recovery_effects()
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "same-checkpoint distinct V3 publication indeterminate: handoff={:?}, reopen={:?}, effects={}",
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure(),
                indeterminate.recovery_effects(),
            ),
            other => panic!("same-checkpoint distinct V3 sequence must recover: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("two-batch C8/Store custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn pending_wal_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = std::path::PathBuf::from(std::env::var_os(MARKER_ENV).expect("child marker"));
    if let Some(first) = role.to_str().and_then(first::World::of_role) {
        return first::child(&marker, first);
    }
    match role.to_str() {
        Some("second") => crate::production_entry::pending_wal_world::second::child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("second source root")),
            &marker,
        ),
        Some("distinct") => crate::production_entry::pending_wal_world::second::distinct_child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("distinct source root")),
            &marker,
        ),
        Some("successor-first") => crate::production_entry::pending_wal_world::successor::child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
            &marker,
            crate::production_entry::pending_wal_world::successor::FIRST_OBJECT,
            1,
        ),
        Some("successor-first-full") => crate::production_entry::pending_wal_world::successor::child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
            &marker,
            crate::production_entry::pending_wal_world::successor::FIRST_OBJECT,
            crate::production_entry::pending_wal_world::successor::FULL_BATCH,
        ),
        Some("successor-distinct") => crate::production_entry::pending_wal_world::successor::child(
            std::path::Path::new(&std::env::var_os(ROOT_ENV).expect("successor source root")),
            &marker,
            crate::production_entry::pending_wal_world::successor::DISTINCT_OBJECT,
            1,
        ),
        other => panic!("unknown pending V3 child role: {other:?}"),
    }
}
#[cfg(all())]
#[path = "three_batch/mod.rs"]
pub(crate) mod three_batch;
