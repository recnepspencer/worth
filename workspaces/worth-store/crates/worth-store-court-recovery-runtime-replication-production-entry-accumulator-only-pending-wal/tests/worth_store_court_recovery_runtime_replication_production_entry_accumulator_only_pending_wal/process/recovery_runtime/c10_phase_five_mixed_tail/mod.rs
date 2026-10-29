



#[allow(unused_imports)] use crate::c10_phase_five_mixed_tail::*;
use {crate::c10_phase_five_mixed_tail::c10_crash_evidence::ordinary_limits, crate::c10_phase_five_mixed_tail::c10_crash_evidence::prepare_rewrite, crate::c10_phase_five_mixed_tail::c10_crash_evidence::segment_snapshot, crate::c10_phase_five_mixed_tail::c10_crash_evidence::wal_contains_rewrite};
use {std::path::Path, std::path::PathBuf};
use {std::process::Command, std::process::Stdio};
use {std::thread};
use {std::time::Duration, std::time::Instant};

use {worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};
use {worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

#[test]
fn mixed_append_and_rewrite_tail_publishes_the_rewrite_once() {
    for seam in ["mixed-before-wal", "mixed-after-wal"] {
        let (parent, root) = kill_child(seam);
        let expected: &[u8] = b"phase5-mixed-append";
        let killed = segment_snapshot(&root);
        let rewrite_before = wal_contains_rewrite(&root);
        let (first, first_payload) = recover(&root, parent.path());
        let published = segment_snapshot(&root);
        let (second, second_payload) = recover(&root, parent.path());
        assert_eq!(first, second, "{seam}");
        assert_eq!(first_payload, expected, "{seam}");
        assert_eq!(second_payload, expected, "{seam}");
        assert_eq!(published, segment_snapshot(&root), "{seam}");
        if seam == "mixed-before-wal" {
            assert!(!rewrite_before, "{seam}");
            assert!(!wal_contains_rewrite(&root), "{seam}");
            assert_eq!(killed, published, "{seam}");
        } else {
            assert!(rewrite_before, "{seam}");
            assert_ne!(killed, published, "{seam}");
            assert!(published.len() <= killed.len() + 1, "{seam}");
        }
        drop(parent);
    }
}

#[test]
fn freshly_admitted_rewrite_keeps_the_published_append() {
    for seam in [
        "competing-then-after-wal",
        "competing-then-after-data",
        "competing-then-during-root",
    ] {
        let (parent, root) = kill_child(seam);
        let expected: &[u8] = b"phase5-member-a";
        let killed = segment_snapshot(&root);
        let (first, first_payload) = recover(&root, parent.path());
        let published = segment_snapshot(&root);
        let (second, second_payload) = recover(&root, parent.path());
        assert_eq!(first, second, "{seam}");
        assert_eq!(first_payload, expected, "{seam}");
        assert_eq!(second_payload, expected, "{seam}");
        assert_eq!(published, segment_snapshot(&root), "{seam}");
        assert!(
            published.len() <= killed.len() + 1,
            "{seam} published more than one rewritten generation"
        );
        drop(parent);
    }
}

#[test]
#[ignore = "launched by the phase 5 mixed-tail parent"]
fn c10_mixed_child_parks_at_seam() {
    let marker = std::path::PathBuf::from(std::env::var_os(CHILD_MARKER).expect("phase 5 mixed marker"));
    let seam = std::env::var(CHILD_SEAM).expect("phase 5 mixed seam");
    let world =
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c10-phase5-mixed-tail").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x91; 32], b"phase5-mixed-source");
    checkpoint_source(&world);
    match seam.as_str() {
        "mixed-before-wal" | "mixed-after-wal" => {
            publish_append(&world, &marker, [0x92; 32], b"phase5-mixed-append");
        }
        "competing-then-after-wal" | "competing-then-after-data" | "competing-then-during-root" => {
            finish_append_ahead_of_stale_rewrite(&world, &marker)
        }
        other => panic!("unknown mixed seam {other}"),
    }
    let gate = world
        .serving()
        .certification_pause_physical_mutation_at(rewrite_checkpoint(&seam));
    let rewrite = prepare_rewrite(&world, [0x94; 32]).start();
    assert!(gate.await_arrival(), "rewrite did not reach {seam}");
    std::mem::forget(rewrite);
    std::mem::forget(gate);
    std::fs::write(marker.join("reached"), b"1").unwrap();
    loop {
        std::thread::park();
    }
}
