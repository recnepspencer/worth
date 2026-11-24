//! Relational publication, change streams, and settlement.
#![forbid(unsafe_code)]
#![deny(unreachable_patterns)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]

#[doc(hidden)]
#[path = "internal_port.rs"]
pub mod internal_port;

use internal_port::*;

#[cfg(test)]
#[test]
fn owned_carriers_stay_within_declared_inline_ceilings() {
    let mut census =
        worth_relational_contracts::internal_port::inline_type_sizes::census::Census::default();
    authority::commit::inline_type_budget::collect(&mut census);
    mvcc::publication::inline_type_budget::collect(&mut census);
    assert!(
        census.exceeded.is_empty(),
        "inline type ceiling exceeded: {:?}",
        census.exceeded
    );
}
