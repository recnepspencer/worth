use super::*;
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::PartitionId;

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::new(7), slot, 1)
}

fn key(slot: u64) -> WorkflowInstanceProgressKey {
    WorkflowInstanceProgressKey::new(3, entity(slot), entity(90))
}

fn progress(head: u64) -> WorkflowInstanceProgress {
    WorkflowInstanceProgress {
        head: entity(head),
        next_occurrence: 2,
        retry_counts: BTreeMap::from([(
            (entity(1), ApplicationWorkflowControlOutcome::Completed),
            2,
        )]),
    }
}

#[test]
fn exact_revision_reuses_and_changed_revision_reconstructs() {
    let mut retention = WorkflowInstanceProgressRetention::new(4096);
    let revision = Some(VersionId::new(11));
    retention
        .retain(key(1), revision, progress(4), 2)
        .expect("bounded progress retains");

    assert_eq!(retention.reuse(key(1), revision), Some(progress(4)));
    assert_eq!(retention.reuse(key(1), Some(VersionId::new(12))), None);
    assert_eq!(retention.counters().warm_hits(), 1);
    assert_eq!(retention.counters().cold_misses(), 1);
    assert_eq!(
        retention.counters().cold_reconstruction_transition_visits(),
        2
    );
}

#[test]
fn exact_owner_result_advances_once_and_duplicate_delivery_is_idempotent() {
    let mut retention = WorkflowInstanceProgressRetention::new(4096);
    let source_revision = Some(VersionId::new(13));
    let committed_revision = Some(VersionId::new(14));
    let source = progress(4);
    let advanced = progress(5);
    retention
        .retain(key(1), source_revision, source.clone(), 2)
        .expect("source progress retains");

    retention
        .advance(
            key(1),
            source_revision,
            &source,
            committed_revision,
            advanced.clone(),
        )
        .expect("continuous owner result advances progress");
    retention
        .advance(
            key(1),
            source_revision,
            &source,
            committed_revision,
            advanced.clone(),
        )
        .expect("duplicate owner result reuses the exact advance");

    assert_eq!(retention.reuse(key(1), committed_revision), Some(advanced));
    assert_eq!(retention.counters().incremental_advances(), 1);
    assert_eq!(retention.counters().incremental_replays(), 1);
}

#[test]
fn gap_foreign_basis_and_eviction_cannot_advance_progress() {
    let mut retention = WorkflowInstanceProgressRetention::new(4096);
    let source_revision = Some(VersionId::new(15));
    let source = progress(6);
    retention
        .retain(key(2), source_revision, source.clone(), 2)
        .expect("source progress retains");

    assert_eq!(
        retention.advance(
            key(2),
            Some(VersionId::new(99)),
            &source,
            Some(VersionId::new(16)),
            progress(7),
        ),
        Err(WorkflowInstanceProgressRetentionDenial::RevisionCollision)
    );
    assert_eq!(
        retention.advance(
            key(3),
            source_revision,
            &source,
            Some(VersionId::new(16)),
            progress(7),
        ),
        Err(WorkflowInstanceProgressRetentionDenial::ContinuityUnavailable)
    );
    assert_eq!(
        retention.advance(
            key(2),
            source_revision,
            &source,
            source_revision,
            progress(7),
        ),
        Err(WorkflowInstanceProgressRetentionDenial::RevisionCollision)
    );
    assert_eq!(retention.reuse(key(2), source_revision), Some(source));
    assert_eq!(retention.counters().incremental_misses(), 1);
}

#[test]
fn equal_revision_collision_fails_closed() {
    let mut retention = WorkflowInstanceProgressRetention::new(4096);
    let revision = Some(VersionId::new(21));
    retention
        .retain(key(2), revision, progress(5), 2)
        .expect("first progress retains");

    assert_eq!(
        retention.retain(key(2), revision, progress(6), 2),
        Err(WorkflowInstanceProgressRetentionDenial::RevisionCollision)
    );
    assert_eq!(retention.counters().denials(), 1);
}

#[test]
fn stale_reconstruction_cannot_replace_newer_progress() {
    let mut retention = WorkflowInstanceProgressRetention::new(4096);
    let newer = progress(7);
    retention
        .retain(key(2), Some(VersionId::new(23)), newer.clone(), 3)
        .expect("newer progress retains");

    assert_eq!(
        retention.retain(key(2), Some(VersionId::new(22)), progress(6), 2),
        Err(WorkflowInstanceProgressRetentionDenial::RevisionCollision)
    );
    assert_eq!(
        retention.reuse(key(2), Some(VersionId::new(23))),
        Some(newer)
    );
}

#[test]
fn least_recently_used_progress_evicts_within_the_shard_budget() {
    let sample = progress(7);
    let one_entry = std::mem::size_of::<WorkflowInstanceProgressKey>()
        .saturating_add(std::mem::size_of::<RetainedWorkflowInstanceProgress>())
        .saturating_add(sample.retained_charge_bytes());
    let mut retention = WorkflowInstanceProgressRetention::new(one_entry);
    retention
        .retain(key(3), Some(VersionId::new(31)), sample, 2)
        .expect("first progress fits exactly");
    retention
        .retain(key(4), Some(VersionId::new(32)), progress(8), 2)
        .expect("second progress evicts the first");

    assert_eq!(retention.reuse(key(3), Some(VersionId::new(31))), None);
    assert_eq!(
        retention.reuse(key(4), Some(VersionId::new(32))),
        Some(progress(8))
    );
    assert_eq!(retention.counters().evictions(), 1);
    assert!(retention.counters().retained_charge_bytes() <= one_entry);
}

#[test]
fn branch_release_removes_only_the_retired_occurrence() {
    let mut retention = WorkflowInstanceProgressRetention::new(8192);
    let other = WorkflowInstanceProgressKey::new(4, entity(5), entity(90));
    retention
        .retain(key(5), Some(VersionId::new(41)), progress(8), 2)
        .expect("retired branch progress retains");
    retention
        .retain(other, Some(VersionId::new(42)), progress(9), 2)
        .expect("live branch progress retains");

    retention.release_branch(3);

    assert_eq!(retention.reuse(key(5), Some(VersionId::new(41))), None);
    assert_eq!(
        retention.reuse(other, Some(VersionId::new(42))),
        Some(progress(9))
    );
    assert_eq!(retention.counters().releases(), 1);
}
