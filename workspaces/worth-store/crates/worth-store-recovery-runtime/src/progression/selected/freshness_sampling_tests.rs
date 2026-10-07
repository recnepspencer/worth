//! Genuine selected checkpoint/WAL sampling denies live native pressure before copying.

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryStaticConfiguration,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    num::NonZeroU64,
    path::{Path, PathBuf},
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalOperationAllocationScope as Scope,
    PhysicalRecoveryRejoinResidentDenial, PhysicalResidencyDimension as Dimension,
    StoreRecoveryBindingSampleAllocationDenial,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

const ORIGINAL: u64 = 16 << 20;

#[test]
fn selected_checkpoint_and_wal_sampling_denies_live_recovery_pressure_before_copying() {
    std::thread::Builder::new()
        .name("selected-freshness-native-pressure".to_owned())
        .stack_size(16 << 20)
        .spawn(run_sampling_pressure)
        .unwrap()
        .join()
        .unwrap();
}

fn run_sampling_pressure() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("freshness-sampling-pressure")
        .unwrap();
    let acknowledgment =
        canonical_physical_mutation_acknowledgment(&world, [0xe1; 32], b"checkpoint-covered");
    assert_ne!(acknowledgment.request_fingerprint().bytes(), [0; 32]);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xe2; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("ordinary production checkpoint must admit");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    // Actual issued-key preparation, WAL append and barrier leave a genuine tail.
    canonical_durable_wal_attempt_without_execution(&world, [0xe3; 32], b"wal-tail-member");
    let retained = world.retained_root();
    let root = retained.path();
    drop(world);
    let media_before = family_digests(root);
    let mut selected = recovery_request(root)
        .admit()
        .unwrap()
        .discover()
        .unwrap()
        .select()
        .unwrap();
    assert!(selected.checkpoint_identity().is_some());
    assert!(
        selected
            .selection
            .checkpoint()
            .unwrap()
            .checkpoint()
            .footer()
            .binding_record_count()
            > 0,
        "actual checkpoint contains the completed owner's retained binding, not only empty custody"
    );
    assert!(selected.wal_frame_count() > 0);
    let owner = selected.coordination.owner_mut();
    let observer = owner.certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let active = observer.snapshot().for_dimension(dimension).active_units();
    assert!(
        owner.owned_recovery_heap_bytes().unwrap() > 0,
        "real Shared checkpoint and retained binding basis exist before sampling"
    );
    let held_bytes = ORIGINAL.checked_sub(active).unwrap() - 1;
    let held = owner
        .certification_begin_recovery_allocation(NonZeroU64::new(held_bytes).unwrap())
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    assert_eq!(before.active_units(), ORIGINAL - 1);
    let Err(PhysicalRecoveryOutcome::Blocked(blocked)) = selected.plan() else {
        panic!("sample backing must deny before planning copies or recovery effects");
    };
    assert_eq!(blocked.recovery_effects(), 0);
    let Some(PhysicalRecoveryPlanningDenial::BindingSamplingAllocation(
        StoreRecoveryBindingSampleAllocationDenial::Backing {
            requested,
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
        },
    )) = &blocked.evidence().planning_denial
    else {
        panic!(
            "exact original/native sampling cause required: {:?}",
            blocked.evidence().planning_denial
        );
    };
    assert!(*requested > 1);
    assert_eq!(*required, ORIGINAL - 1 + *requested);
    assert_eq!(*admitted, ORIGINAL);
    let after = observer.snapshot().for_dimension(dimension);
    assert_eq!(after.admissions(), before.admissions());
    assert_eq!(after.admitted_units(), before.admitted_units());
    assert_eq!(after.denials(), before.denials() + 1);
    let observation_bytes = blocked.evidence().integrity_observations.charged_bytes();
    assert!(
        observation_bytes > 0,
        "blocked diagnostics retain the actual WAL observation owner"
    );
    assert_eq!(
        after.active_units(),
        held_bytes + observation_bytes,
        "blocked selected material releases checkpoint and binding storage but retains diagnostics"
    );
    assert_eq!(family_digests(root), media_before);
    drop(held);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        observation_bytes,
        "only the blocked outcome's shared observations retain Recovery storage"
    );
    drop(blocked);
    let disposed = observer.snapshot().for_dimension(dimension);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());

    let outcome = crate::WorthStoreRecovery::recover(recovery_request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("healthy retry must recover real unchanged media: {outcome:?}");
    };
    assert!(!handoff.freshness_sample().operations().is_empty());
    assert!(!handoff.freshness_sample().wal_members().is_empty());
    let allocations = handoff.core().certification_residency_allocations();
    drop(handoff);
    let disposed = allocations.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

fn recovery_request(root: &Path) -> PhysicalRecoveryOpenRequest {
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let limits = PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: 64 << 20,
        manifest_entries: 4096,
        wal_segments: 64,
        wal_frames: 4096,
        wal_bytes: 64 << 20,
        redo_targets: 4096,
        redo_bytes: 64 << 20,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: 64 << 20,
        recovery_memory_bytes: ORIGINAL,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4096,
        cleanup_bytes: 64 << 20,
        observation_bytes: 64 << 20,
    })
    .unwrap();
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_owned(), configuration.clone(), limits)
            .unwrap();
    let backend = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(root.to_owned(), configuration, backend, limits, authority)
}

fn family_digests(root: &Path) -> BTreeMap<PathBuf, [u8; 32]> {
    fn visit(directory: &Path, files: &mut BTreeMap<PathBuf, [u8; 32]>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, files);
            } else {
                files.insert(
                    path.clone(),
                    Sha256::digest(std::fs::read(path).unwrap()).into(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(&root.join("families"), &mut files);
    files
}
