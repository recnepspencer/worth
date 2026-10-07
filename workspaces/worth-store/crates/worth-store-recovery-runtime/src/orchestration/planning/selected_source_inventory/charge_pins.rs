//! Each read of a root's pages takes that root's charge: lent to every read
//! but the last, which spends it. A wrapper changed to take no charge, or to
//! take it otherwise, no longer coerces here, and the crate's tests do not
//! compile.

use super::super::manifest_entry_budget::{ChargeToken, ManifestEntryBudget};
use super::{
    observe_headers, observe_routes_held, observe_routes_with_budget, observe_with_budget,
    InventoryHeaders, InventorySegments, ResidentAllowance,
};

const _: () = {
    let _: fn(_, _, _, &ChargeToken, _) -> _ = observe_headers;
    let _: fn(_, _, _, _, &ChargeToken, &mut ManifestEntryBudget, _) -> _ =
        InventoryHeaders::observe_segments;
    let _: fn(_, _, _, _, &ChargeToken, &mut ManifestEntryBudget, _) -> _ =
        InventorySegments::observe_free_entries;
    let _: fn(_, _, _, ChargeToken, &mut ManifestEntryBudget, _) -> _ = observe_with_budget;
    let _: fn(_, _, _, ChargeToken, &mut ManifestEntryBudget, _) -> _ = observe_routes_with_budget;
    let _: fn(_, _, _, ChargeToken, &mut ManifestEntryBudget, _, &mut ResidentAllowance) -> _ =
        observe_routes_held;
};
