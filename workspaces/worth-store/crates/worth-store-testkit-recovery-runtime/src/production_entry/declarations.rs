pub use std::path::Path;
pub use std::process::Command;
pub use {std::num::NonZeroU16, std::num::NonZeroU64};

pub use worth_store_authority::require_current_store_authority;
pub use worth_store_blob_chunks::BlobChunkSize;
pub use {
    worth_foundational::aspects, worth_foundational::AspectValue,
    worth_foundational::ContractValidationInput, worth_foundational::InternedString,
    worth_foundational::ScalarAspectType,
};
pub use {worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome};
pub use {
    worth_store::aspect_native::StoreAspectAuthorityInput,
    worth_store::aspect_native::StoreAspectBoundaryFact,
    worth_store::aspect_native::StoreAspectIdentity,
    worth_store::aspect_native::StorePhysicalAuthorityWitness,
    worth_store::aspect_native::StorePhysicalBoundaryWitness,
    worth_store::aspect_native::ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};
pub use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestFailure,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};
pub use {
    worth_store_security::admit_store_security_scope,
    worth_store_security::StoreAuthenticityRequirement,
    worth_store_security::StoreAuthenticityRequirementClass,
    worth_store_security::StoreCustodyPosture, worth_store_security::StoreKeyScope,
    worth_store_security::StoreKeyVersionPosture,
    worth_store_security::StoreSecurityScopeAdmissionExpectation,
    worth_store_security::StoreSecurityScopeAdmissionRequest,
    worth_store_security::StoreTenantScope,
};
pub use {
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub use crate::production_entry::blob_scope_admission::admitted_blob_scope;

pub use {
    crate::production_entry::ordered_journals::canonical_journal,
    crate::production_entry::ordered_journals::canonical_journal_oracle,
};

pub fn run_entry(root: &Path) -> std::process::Output {
    std::process::Command::new(worth_store_testkit_executables::physical_store_recover())
        .arg(root)
        .arg("--bounded-profile=c8-phase2-admission-v1")
        .output()
        .expect("run production recovery entry")
}

pub fn runtime_text(stderr: &str) -> &str {
    stderr
        .split_once(" runtime ")
        .expect("runtime prefix")
        .1
        .split_once(" at root generation")
        .expect("runtime suffix")
        .0
}

pub fn initialized_recovery_world(label: &str) -> PhysicalResidencyStoreWorld {
    let _ = crate::setup_observation::record("initialized-recovery-world", &|inputs| {
        inputs.bytes("label", label.as_bytes());
        inputs.bytes("mutation-key", &[0x41; 32]);
        inputs.bytes("mutation-payload", b"production-entry-redo");
        inputs.bytes("checkpoint-key", &[0x42; 32]);
        inputs.unsigned("checkpoint-deadline-ms", 5_000);
    });
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x41; 32], b"production-entry-redo");
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x42; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(5_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("production entry checkpoint admission must succeed")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    world
}

pub fn test_limits() -> worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration {
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
