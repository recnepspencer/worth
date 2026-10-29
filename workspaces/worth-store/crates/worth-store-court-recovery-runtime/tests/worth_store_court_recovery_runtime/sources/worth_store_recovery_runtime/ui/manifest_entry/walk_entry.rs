//! The runtime's walk entry, as the compile-fail cases see it: the real leaf
//! module that owns the manifest-entry budget, included whole, under the one
//! parent allowed to build a budget. The wrappers below have the shapes of the
//! runtime wrappers that issue entry-charged reads: lent to all but a root's
//! last read, which spends the charge.

#[path = "../../../src/orchestration/planning/page_observation/walk_entry/manifest_entry_budget.rs"]
pub mod manifest_entry_budget;

use manifest_entry_budget::{pays_for, spend, ChargeToken, ManifestEntryBudget};

/// The walk's one budget.
pub fn walk_budget(admitted: u64) -> ManifestEntryBudget {
    ManifestEntryBudget::declared(admitted, 0)
}

/// A read of root `generation` that is not its last: `charge` is lent.
pub fn read_page(charge: &ChargeToken, generation: u64) {
    pays_for(charge, generation);
}

/// The last read of root `generation`: it takes `charge` and spends it.
pub fn read_root(charge: ChargeToken, generation: u64) {
    read_page(&charge, generation);
    spend(charge, generation);
}
