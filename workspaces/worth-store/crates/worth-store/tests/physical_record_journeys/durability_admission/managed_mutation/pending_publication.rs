use worth_store::physical_runtime::{
    PhysicalMutationIndeterminateStage, PhysicalMutationOutcome,
    PhysicalMutationProvenNoEffectCause,
};
use worth_store_physical_backend::MediaOperationRole;

use super::*;

#[test]
fn pending_publication_is_registered_before_the_first_wal_effect() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let before = serving.media_counters();
    let gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::BeforeWalAppend,
    );
    let handle = prepare(&serving, placement, [19; 32], b"pending-before-wal").start();
    assert!(gate.await_arrival());
    assert_eq!(serving.certification_pending_publication_count(), 1);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        before.attempts_for(MediaOperationRole::PositionedWrite)
    );
    gate.release();
    handle.wait();
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            > before.attempts_for(MediaOperationRole::PositionedWrite)
    );
    serving.close();
}

#[test]
fn indeterminate_wal_append_with_no_effects_keeps_the_publication_hold() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    serving.certification_fail_next_wal_member_before_effect();
    match prepare(&serving, placement, [21; 32], b"wal-not-started").execute() {
        PhysicalMutationOutcome::Indeterminate(fate) => {
            assert_eq!(fate.stage(), PhysicalMutationIndeterminateStage::WalAppend);
            assert_eq!(fate.completed_effect_count(), 0);
        }
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!(
                "unstarted WAL append must stay indeterminate: {:?}",
                fate.cause()
            )
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("unstarted WAL append must not complete")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 1);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    match prepare(&serving, placement, [22; 32], b"blocked-by-indeterminate").execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                PhysicalMutationProvenNoEffectCause::ScopeConflict
            );
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("an indeterminate publication must block the next root change")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("the blocked successor must prove no effect")
        }
    }
    serving.close();
}

#[test]
fn a_wal_durable_publication_blocks_the_next_root_change_without_a_second_reservation() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let first = prepare(&serving, placement, [21; 32], b"durable-predecessor").start();
    assert!(gate.await_arrival());
    assert_eq!(serving.certification_pending_publication_count(), 1);
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    let second = prepare(&serving, placement, [22; 32], b"blocked-successor").start();
    match second.wait() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::ScopeConflict
            );
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {
            panic!("the successor must not complete while a publication is pending")
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("the successor must be refused before its WAL effect")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 1);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    gate.release();
    first.wait();
    assert_eq!(serving.certification_pending_publication_count(), 0);
    serving.close();
}
