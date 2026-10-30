#![allow(dead_code)]

#[path = "manifest_entry/walk_entry.rs"]
mod walk_entry;

use walk_entry::manifest_entry_budget::ChargeTarget;

/// A charge pays at least one entry: nothing mints a token that paid nothing.
fn charge_nothing() {
    let mut budget = walk_entry::walk_budget(10);
    let charge = budget.charge(0, ChargeTarget::root(7)).unwrap();
    let _ = walk_entry::read_root(charge, 7);
}

fn main() {}
