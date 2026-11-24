//! Branch layout evidence using the shared Relational census.
use crate::inline_type_sizes::census::Census;

#[test]
fn owned_carriers_stay_within_declared_inline_ceilings() {
    let mut census = Census::default();
    crate::branch::inline_type_budget::collect(&mut census);
    crate::history::inline_type_budget::collect(&mut census);
    assert!(
        census.exceeded.is_empty(),
        "inline type ceiling exceeded: {:?}",
        census.exceeded,
    );
}
