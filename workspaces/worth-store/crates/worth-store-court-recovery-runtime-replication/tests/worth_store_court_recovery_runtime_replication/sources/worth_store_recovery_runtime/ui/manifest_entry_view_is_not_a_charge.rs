#![allow(dead_code)]

#[path = "manifest_entry/walk_entry.rs"]
mod walk_entry;

use walk_entry::manifest_entry_budget::{ChargeTarget, ViewEntryCap, ROOT_ENTRY};

/// A view refuses past its cap, but charges nothing and pays for no read.
fn read_under_a_view() {
    let budget = walk_entry::walk_budget(10);
    let mut view = ViewEntryCap::of(&budget);
    let _ = walk_entry::read_root(view, 7);
    let _ = view.charge(ROOT_ENTRY, ChargeTarget::root(7));
}

fn main() {}
