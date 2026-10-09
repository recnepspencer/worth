#![allow(dead_code)]

#[path = "manifest_entry/walk_entry.rs"]
mod walk_entry;

use walk_entry::manifest_entry_budget::{
    ChargeTarget, EntriesCharged, ManifestEntryAuthority,
};
use worth_proof::Performed;

/// A wrapper cannot be called without a charge: none is made but by the
/// budget.
fn read_unpaid() {
    let forged = Performed::<EntriesCharged, ManifestEntryAuthority, ChargeTarget>::record(
        &ManifestEntryAuthority::witness(),
        ChargeTarget::root(7),
    );
    let _ = walk_entry::read_root(forged, 7);
}

fn main() {}
