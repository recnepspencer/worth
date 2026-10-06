//! Each read of the candidate root's pages takes the root's charge: lent to
//! every read but the free-space tree, the last, which spends it. A wrapper
//! changed to take no charge, or to take it otherwise, no longer coerces
//! here, and the crate's tests do not compile.

use super::super::manifest_entry_budget::ChargeToken;
use super::{artifact_read, free_space, root_manifest, root_routing, segment_membership};

const _: () = {
    let _: fn(_, _, _, &ChargeToken, _) -> _ = artifact_read::read;
    let _: fn(_, _, _, &ChargeToken, _, _, _) -> _ = root_manifest::read;
    let _: fn(_, _, _, &ChargeToken, _, _, _, _, _, _) -> _ = root_routing::read;
    let _: fn(_, _, _, &ChargeToken, _, _, _, _, _, _) -> _ = segment_membership::read;
    let _: fn(_, _, _, ChargeToken, _, _, _, _, _, _) -> _ = free_space::read;
};
