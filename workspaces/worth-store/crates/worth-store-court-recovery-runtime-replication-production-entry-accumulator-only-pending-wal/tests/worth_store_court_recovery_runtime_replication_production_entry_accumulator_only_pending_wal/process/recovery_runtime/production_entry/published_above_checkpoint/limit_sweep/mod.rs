//! A recovery admitted under less than its world needs is told which limit
//! ran out and the value it was admitted under, at every limit below the
//! need and whichever phase met it. None of them is told its media is
//! damaged. The same media answers every probe: a blocked recovery has no
//! effect.
//!
//! Manifest entries, manifest bytes, observation bytes and staging bytes
//! are each swept from one up to the need.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::*;
use {worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::manifest_entry_limit::released_world, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::manifest_entry_limit::IDLE_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::manifest_entry_limit::ORDERED_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::manifest_entry_limit::RELEASED_NEED};

#[test]
fn every_limit_under_the_need_of_published_objects_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        idle_world,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::every(IDLE_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::History::Ordered),
        "published above the checkpoint",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II Milestone 15"]
fn every_limit_under_the_need_of_a_historical_release_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        released_world,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::every(RELEASED_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::History::Ordered),
        "released above the checkpoint",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II Milestone 15"]
fn every_limit_under_the_need_of_a_release_above_a_checkpoint_head_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history::terminal_successor_of_a_checkpoint_head,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::every(HEADED_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::History::Ordered),
        "released above a checkpoint head",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II Milestone 15"]
fn every_limit_under_the_need_of_a_certified_release_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::worlds::certified_release,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::every(CERTIFIED_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::History::Ordered).unstaged(),
        "certified release",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II Milestone 15"]
fn every_limit_under_the_need_of_completed_history_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::worlds::completed_history,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::strided(COMPLETED_NEED, 16),
        "completed history",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II Milestone 15"]
fn every_limit_under_the_need_of_a_pending_tier_release_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worth_store_testkit_recovery_runtime::production_entry::tier_release_pending_wal::kill_producer_after_descriptor_wal,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::every(TIER_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::History::Unordered),
        "pending tier release",
    );
}

#[test]
fn every_limit_under_the_need_of_a_killed_rewrite_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::moved_media::killed_rewrite,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::every(REWRITE_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::History::Unordered),
        "killed rewrite",
    );
}

#[test]
fn every_limit_under_the_need_of_a_killed_copy_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::moved_media::killed_copy,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::every(COPY_NEED, worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::History::Ordered),
        "killed copy",
    );
}

#[test]
#[ignore = "release-limit-sweeps: release and reclaim report Absent until Part II Milestone 15"]
fn every_limit_under_the_need_of_a_pending_successor_is_reported_as_that_limit() {
    assert_limits_are_reported_as_limits(
        worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history::ordered_release_above_a_head_checkpoint,
        worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::entries::Need::strided(ORDERED_NEED, 16),
        "pending successor of an ordered release",
    );
}
#[path = "moved_media/mod.rs"]
pub(crate) mod moved_media;
