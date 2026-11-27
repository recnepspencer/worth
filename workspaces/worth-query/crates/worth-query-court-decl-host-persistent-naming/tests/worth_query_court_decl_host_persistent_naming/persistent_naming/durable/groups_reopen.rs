//! Group names across Store reopen.
//!
//! Replaying the Journal and reopening from a `CHK24` checkpoint both preserve
//! every branch's group state (Store M4.3f / recnepspencer/worth#440), so held
//! loop names, `group_name_of` and `resolve_group` answer identically across
//! both Journal-only and checkpoint reopens.
use super::super::outputs::World;
use super::super::world::home;
use super::admission::named_world;
use super::*;

fn reopened_serves_the_same_groups(checkpoint: bool) {
    let mut world = World::open_directory(home::location());
    let held = named_world(&world, if checkpoint { "gc" } else { "gj" });
    assert!(!held.loops.is_empty());
    crash_and_compare(&mut world, &held, "groups", checkpoint);
}

#[test]
fn group_names_survive_a_journal_reopen() {
    reopened_serves_the_same_groups(false);
}

#[test]
fn group_names_survive_a_checkpoint_reopen() {
    reopened_serves_the_same_groups(true);
}
