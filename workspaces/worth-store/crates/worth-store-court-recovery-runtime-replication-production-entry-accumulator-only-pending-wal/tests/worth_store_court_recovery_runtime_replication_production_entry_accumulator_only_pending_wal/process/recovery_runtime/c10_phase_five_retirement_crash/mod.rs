


#[allow(unused_imports)] use crate::c10_phase_five_retirement_crash::*;
use {crate::c10_phase_five_retirement_crash::c10_crash_evidence::ordinary_limits, crate::c10_phase_five_retirement_crash::c10_crash_evidence::prepare_rewrite, crate::c10_phase_five_retirement_crash::c10_crash_evidence::segment_snapshot, crate::c10_phase_five_retirement_crash::c10_crash_evidence::wal_contains_rewrite};
use {std::path::Path, std::path::PathBuf};
use {std::process::Command, std::process::Stdio};
use {std::thread};
use {std::time::Duration, std::time::Instant};

use {worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};
use {worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

#[test]
fn killed_retirement_intent_survives_a_fresh_recovery_and_second_reopen() {
    let (parent, root) = kill_child("before-delete");
    let present = segment_names(&root);
    let first = recovered_retirements(&root);
    let second = recovered_retirements(&root);
    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    assert!(first[0].3 > 0);
    let name = segment_name(first[0].1, first[0].2);
    assert!(present.iter().any(|file| file == &name));
    assert!(segment_names(&root).iter().any(|file| file == &name));
    drop(parent);
}

#[test]
fn killed_retirement_after_unlink_keeps_the_obligation_until_completion() {
    let (parent, root) = kill_child("after-unlink");
    let present = segment_names(&root);
    let first = recovered_retirements(&root);
    let second = recovered_retirements(&root);
    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    assert!(first[0].3 > 0);
    let name = segment_name(first[0].1, first[0].2);
    assert!(present.iter().all(|file| file != &name));
    assert!(segment_names(&root).iter().all(|file| file != &name));
    drop(parent);
}

#[test]
fn killed_retirement_after_completion_does_not_resurrect_the_obligation() {
    let obligation = kill_and_recover("after-completion");
    assert!(
        obligation.is_empty(),
        "a durable completion must not come back as unfinished cleanup"
    );
}

#[test]
fn killed_rewrite_behind_a_durable_append_leaves_no_rewrite_effect() {
    let (parent, root) = kill_child("competing-append");
    let killed = segment_snapshot(&root);
    assert!(
        !wal_contains_rewrite(&root),
        "a blocked rewrite must not append redo before the kill"
    );
    let first = recovered_retirements(&root);
    let published = segment_snapshot(&root);
    let second = recovered_retirements(&root);
    assert_eq!(first, second);
    assert!(
        first.is_empty(),
        "the blocked rewrite must not retire a generation"
    );
    assert_ne!(
        killed, published,
        "recovery must publish the append that was already WAL-durable"
    );
    assert!(
        !wal_contains_rewrite(&root),
        "recovery must not invent rewrite redo for the blocked writer"
    );
    assert_eq!(published, segment_snapshot(&root));
    drop(parent);
}

#[test]
#[ignore = "launched by the phase 5 retirement parent"]
fn c10_retirement_child_parks_at_seam() {
    let marker = std::path::PathBuf::from(std::env::var_os(CHILD_MARKER).expect("phase 5 marker"));
    let seam = std::env::var(CHILD_SEAM).expect("phase 5 seam");
    let world =
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c10-phase5-retirement-kill").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    canonical_physical_mutation_acknowledgment(&world, [0x71; 32], b"phase5-retirement-source");
    if seam != "competing-append" {
        publish_rewrite(&world);
    }
    match seam.as_str() {
        "before-delete" => park_retirement_until_killed(&world, &marker, 1),
        "after-unlink" => park_retirement_until_killed(&world, &marker, 2),
        "after-completion" => {
            world
                .serving()
                .retire_displaced_segment()
                .expect("retirement reaches a durable completion");
        }
        "competing-append" => park_behind_durable_append(&world),
        other => panic!("unknown seam {other}"),
    }
    std::fs::write(marker.join("reached"), b"1").unwrap();
    loop {
        std::thread::park();
    }
}
