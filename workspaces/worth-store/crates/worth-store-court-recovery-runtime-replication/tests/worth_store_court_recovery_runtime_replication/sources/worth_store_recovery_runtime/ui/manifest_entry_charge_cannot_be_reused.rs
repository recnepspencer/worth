#![allow(dead_code)]

#[path = "manifest_entry/walk_entry.rs"]
mod walk_entry;

use walk_entry::manifest_entry_budget::{ChargeTarget, ROOT_ENTRY};

/// One charge pays for one read: a consumed token cannot pay for a second.
fn read_twice() {
    let mut budget = walk_entry::walk_budget(10);
    let charge = budget
        .charge(ROOT_ENTRY, ChargeTarget::root(7))
        .unwrap();
    let _ = walk_entry::read_root(charge, 7);
    let _ = walk_entry::read_root(charge, 7);
}

fn main() {}
