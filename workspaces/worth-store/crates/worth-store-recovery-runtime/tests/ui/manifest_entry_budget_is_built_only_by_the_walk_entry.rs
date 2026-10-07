#![allow(dead_code)]

#[path = "manifest_entry/walk_entry.rs"]
mod walk_entry;

use walk_entry::manifest_entry_budget::ManifestEntryBudget;

/// A walk has one budget: no module but the walk entry builds another.
fn second_budget() {
    let _ = ManifestEntryBudget::declared(10, 0);
}

fn main() {}
