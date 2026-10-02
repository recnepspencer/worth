//! Real checkpoint/WAL input with native pressure at the WAL observation boundary.

use super::*;
use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
};
use crate::orchestration::RecoveryCoordination;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    num::NonZeroU64,
    path::{Path, PathBuf},
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    ObservedWalArtifact, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalOperationAllocationScope as Scope, PhysicalRecoveryFreshnessPort,
    PhysicalRecoveryObservationAllocationDenial, PhysicalRecoveryRejoinResidentDenial,
    PhysicalResidencyDimension as Dimension, QualifiedRecoveryFilesystemMedia,
    RecoveryWalArtifactView, RecoveryWalReadFailureView,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

const ORIGINAL: u64 = 16 << 20;

mod canonical_pressure;
mod diagnostic_pressure;
mod selection_pressure;

#[test]
fn real_wal_observation_denies_before_read_and_shared_input_retains_native_backing() {
    std::thread::Builder::new()
        .name("wal-discovery-native-pressure".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

fn run() {
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("wal-discovery-pressure").unwrap();
    let acknowledgment =
        canonical_physical_mutation_acknowledgment(&world, [0xf1; 32], b"checkpoint-covered");
    assert_ne!(acknowledgment.request_fingerprint().bytes(), [0; 32]);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xf2; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("ordinary checkpoint must admit");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0xf3; 32], b"wal-tail-member");
    let retained_root = world.retained_root();
    let root = retained_root.path();
    drop(world);
    let media_before = family_digests(root);
    let entry_count = std::fs::read_dir(root.join("families/wal"))
        .unwrap()
        .count();
    let roster_bytes = (entry_count * std::mem::size_of::<ObservedWalArtifact>()) as u64;
    assert!(
        roster_bytes > 1,
        "real WAL directory supplies retained result slots"
    );

    let limits = limits();
    let (payload_gate, schedule) = diagnostic_pressure::payload_pause();
    let qualified =
        QualifiedRecoveryFilesystemMedia::qualify_existing_for_certification(root, schedule)
            .unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let mut media = qualified.admit_persisted_store().unwrap();
    let mut coordination = RecoveryCoordination::fresh(
        &mut media,
        freshness.register_session().unwrap(),
        limits,
        PhysicalRecoveryStaticConfiguration::current().residency_policy(),
        None,
    )
    .unwrap();
    let observer = coordination.owner().certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let mut discovery = media.bounded_discovery(8192, 64 << 20).unwrap();
    let mut counters = PhysicalRecoveryDiscoveryCounters::default();
    let mut trace = crate::integrity_ingress::RecoveryIntegrityIngressTrace::new();
    let mut remaining_manifest = limits.declaration().manifest_bytes;
    // This is the real observation sub-boundary, not a substitute for root
    // selection or plan authority. The final retry below uses the full entry.
    let checkpoint = {
        let mut window = coordination
            .owner_mut()
            .begin_source_read_allocation()
            .unwrap();
        match checkpoint::observe_checkpoint(
            &mut discovery,
            limits,
            &mut remaining_manifest,
            &mut counters,
            &mut trace,
            &mut window,
        ) {
            Ok(checkpoint @ CheckpointDiscovery::Admitted { .. }) => checkpoint,
            Ok(_) => panic!("genuine checkpoint must be admitted"),
            Err(failure) => panic!("checkpoint setup: {:?}", failure.source_denials),
        }
    };
    let prior = observer.snapshot().for_dimension(dimension).active_units();
    assert!(
        prior > 0,
        "actual retained checkpoint/source-root storage precedes WAL pressure"
    );
    let diagnostic_bytes = diagnostic_pressure::deny_before_discovery(
        &mut discovery,
        &mut coordination,
        limits,
        &mut counters,
    );
    assert_eq!(family_digests(root), media_before);
    let named_failure = diagnostic_pressure::deny_payload_and_retain_context(
        root,
        &mut discovery,
        &mut coordination,
        limits,
        &mut counters,
        diagnostic_bytes,
        &payload_gate,
    );
    let error_bytes = named_failure.charged_bytes();
    let prior = prior + error_bytes;

    canonical_pressure::deny_then_retry(&mut discovery, &mut coordination, limits);
    assert_eq!(family_digests(root), media_before);
    let read_before = discovery.counters().wal_bytes_read;
    let (wal, residue, _) =
        match observe_wal(&mut discovery, &mut coordination, limits, &mut counters) {
            Ok(result) => result,
            Err(failure) => panic!("same-owner healthy retry: {:?}", failure.source_denials),
        };
    assert!(residue.is_empty());
    assert!(!wal.rejected);
    assert!(wal.valid_frames > 0);
    assert_eq!(
        discovery.counters().wal_bytes_read - read_before,
        wal.observed_bytes
    );
    assert_eq!(
        wal.integrity_ingress.owner_projection_entries,
        wal.valid_frames
    );
    let shared_segments: Vec<_> = wal
        .admitted
        .cleanup_segments(
            wal.candidates
                .iter()
                .map(|candidate| candidate.inspection().identity()),
        )
        .collect();
    assert!(!shared_segments.is_empty());
    // Count actual native ownership, not the production heap-sizing function.
    // The separate Store owner test independently checks capacities and Arc storage.
    let retained_bytes: u64 = shared_segments
        .iter()
        .map(|segment| {
            segment.charged_bytes()
                + segment
                    .frames()
                    .iter()
                    .map(|frame| frame.charged_bytes())
                    .sum::<u64>()
        })
        .sum();
    assert!(retained_bytes > 0);
    let observation_bytes = wal.integrity_observations.charged_bytes();
    let roster_bytes_retained = wal.admitted.roster_charged_bytes();
    let runtime_bytes = observation_bytes + roster_bytes_retained + wal.candidates.charged_bytes();
    assert!(observation_bytes > 0 && roster_bytes_retained > 0);
    let clone_before = observer.snapshot();
    let diagnostics = wal.integrity_observations.clone();
    assert_eq!(
        diagnostics.wal().as_ptr(),
        wal.integrity_observations.wal().as_ptr()
    );
    assert_eq!(
        observer.snapshot(),
        clone_before,
        "cloning shares storage without native admission"
    );
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        prior + retained_bytes + runtime_bytes
    );

    // Result slots, original names and byte buffers have distinct live backing.
    // Confined paths remain outside this owner. All read storage must
    // coexist with the retained C9 input; dropping one cannot uncharge the other.
    {
        let mut window = coordination
            .owner_mut()
            .begin_source_read_allocation()
            .unwrap();
        let raw = window
            .read_wal_payloads(
                &mut discovery,
                limits.declaration().wal_segments,
                limits.declaration().wal_bytes,
            )
            .unwrap();
        let payload_bytes: u64 = raw
            .artifacts()
            .iter()
            .map(|artifact| artifact.bytes().map_or(0, |bytes| bytes.len() as u64))
            .sum();
        let names = raw
            .artifacts()
            .iter()
            .map(|artifact| artifact.name_heap_bytes() as u64)
            .sum::<u64>();
        assert_eq!(payload_bytes, wal.observed_bytes);
        assert_eq!(raw.artifacts().len(), entry_count);
        assert_eq!(raw.charged_bytes(), roster_bytes + payload_bytes + names);
        assert_eq!(
            observer.snapshot().for_dimension(dimension).active_units(),
            prior + retained_bytes + runtime_bytes + roster_bytes + payload_bytes + names
        );
        drop(raw);
        assert_eq!(
            observer.snapshot().for_dimension(dimension).active_units(),
            prior + retained_bytes + runtime_bytes
        );
    }
    drop(wal);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        prior + retained_bytes + observation_bytes,
        "cleanup shares retain the actual grant after inventory disposal"
    );
    assert!(!shared_segments[0].frames().is_empty());
    drop(shared_segments);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        prior + observation_bytes
    );
    drop(checkpoint);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        observation_bytes + error_bytes
    );
    let media = discovery.finish();
    assert_eq!(media.recovery_effect_count(), 0);
    assert_eq!(family_digests(root), media_before);
    drop(media);
    drop(coordination);
    assert!(!diagnostics.wal().is_empty());
    let retained = observer.snapshot();
    assert_eq!(
        retained.for_dimension(dimension).active_units(),
        observation_bytes + error_bytes
    );
    assert_eq!(
        retained
            .for_dimension(Dimension::OperationBytes)
            .active_units(),
        observation_bytes + error_bytes
    );
    assert_eq!(
        retained.for_dimension(Dimension::TotalBytes).active_units(),
        retained
            .for_dimension(Dimension::MetadataBytes)
            .active_units()
            + observation_bytes
            + error_bytes,
        "the last diagnostic owner retains its pool metadata and observation backing"
    );
    drop(diagnostics);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        error_bytes
    );
    assert!(matches!(
        named_failure.diagnostic(),
        RecoveryWalReadFailureView::Allocation {
            artifact: RecoveryWalArtifactView::WalArtifact(_),
            ..
        }
    ));
    drop(named_failure);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());

    let outcome = crate::WorthStoreRecovery::recover(recovery_request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("full entry retry must plan and hand off genuine media: {outcome:?}");
    };
    assert!(!handoff.freshness_sample().operations().is_empty());
    assert!(!handoff.freshness_sample().wal_members().is_empty());
    let final_owner = handoff.core().certification_residency_allocations();
    drop(handoff);
    let disposed = final_owner.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

fn limits() -> PhysicalRecoveryLimits {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
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
    .unwrap()
}

fn recovery_request(root: &Path) -> PhysicalRecoveryOpenRequest {
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let limits = limits();
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
