//! Group names across Store reopen.
//!
//! Replaying the Journal rebuilds every branch's group state from the
//! persisted group lineage frames, so held loop names, `group_name_of` and
//! `resolve_group` answer identically after a Journal-only reopen.
//!
//! HELD (Phase 7B-1 finding): a reopen from a checkpoint loses group lineage
//! published by any commit that is not a branch head when the checkpoint is
//! captured. Cold checkpoint readmission keeps only head envelopes resident
//! (`native_checkpoint_codec/cold_checkpoint.rs`) and group lineage is
//! installed only from resident envelopes
//! (`checkpoint_restore/cold_directory.rs::readmit_lineage`), so every held
//! group name then refuses `UnknownName` and members answer no group. The
//! fix belongs to the checkpoint manifest and cold restore that Store
//! M4.3f owns (recnepspencer/worth#440); the court below states the required
//! behavior and is held until that lands.
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
#[ignore = "held on recnepspencer/worth#440: cold checkpoint restore drops non-head group lineage (Store M4.3f)"]
fn group_names_survive_a_checkpoint_reopen() {
    reopened_serves_the_same_groups(true);
}
