//! A pending V3 batch that names a per-object predecessor, killed at its
//! durable descriptor WAL. Serving admitted every earlier release without a
//! new checkpoint, so recovery must admit the pending batch from the same
//! checkpoint and ordered history, and then rejoin it as completed history.

#[allow(unused_imports)]
use worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history::*;

/// The checkpoint heads the first object. Its second batch and a first
/// release of another object complete above the checkpoint; the first
/// object's third batch, whose predecessor is that ordered second batch, is
/// killed. Its chain leaves the ordered releases at the checkpoint head.
#[test]
fn pending_successor_of_an_ordered_release_above_a_head_checkpoint_recovers() {
    let world = ordered_release_above_a_head_checkpoint();
    assert_pending_then_completed(&world, "pending successor of an ordered release");
}

/// The checkpoint heads both objects. The first object's next batch completes
/// above it; the other object's next batch, whose predecessor is its own
/// checkpoint head, is killed above that foreign history.
#[test]
fn pending_successor_of_a_checkpoint_head_above_foreign_history_recovers() {
    let world = successor_of_a_checkpoint_head_above_foreign_history();
    assert_pending_then_completed(&world, "pending successor of a checkpoint head");
}

/// The killed batch extends the checkpoint head directly and is terminal, so
/// its head effect replaces the head it replayed with a terminal entry.
#[test]
fn pending_terminal_successor_of_a_checkpoint_head_recovers() {
    let world = terminal_successor_of_a_checkpoint_head();
    assert_pending_then_completed(&world, "pending terminal successor of a checkpoint head");
}

/// No checkpoint heads the object: its first release completed above the
/// checkpoint, and the killed second batch names that ordered release.
#[test]
fn pending_successor_of_a_first_release_without_a_checkpoint_head_recovers() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
    world.kill_successor_of_first_object();
    assert_pending_then_completed(&world, "pending successor of an ordered first release");
}

/// The object checkpointed two resume frontiers during its ingest. The first
/// batch drops only its publication and is headed by a checkpoint. The next
/// batch, killed, drops both frontiers and the chunks beside them; the last
/// one, killed, drops the chunks left and their root node, so the head
/// recovered from it is terminal.
#[test]
fn pending_successors_of_a_generation_with_a_resume_frontier_recover_to_a_terminal_head() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first_with_resume_frontier();
    assert_needs_exactly(&world, FRONTIER_NEED, "frontier object");
    assert_eq!(
        recover_and_checkpoint(&world, "frontier object", [0xd4; 32]),
        1
    );
    world.kill_full_batch_successor_of_first_object();
    assert_pending_then_completed(&world, "pending successor that drops a resume frontier");
    world.kill_full_batch_successor_of_first_object();
    assert_pending_then_completed(&world, "pending terminal successor after a resume frontier");
    let heads = checkpoint_heads(&world, "released frontier object", [0xd5; 32]);
    assert_eq!(heads.len(), 1);
    assert!(
        heads[0].terminal(),
        "the release stayed nonterminal: {heads:?}"
    );
}
