//! Runtime census runner for the declared inline owner budgets.
pub(crate) mod census;
pub(crate) use census::Census;

#[test]
fn owned_carriers_stay_within_declared_inline_ceilings() {
    let mut census = Census::default();
    crate::inline_type_budget::collect(&mut census);
    assert!(
        census.exceeded.is_empty(),
        "inline type ceiling exceeded: {:?}",
        census.exceeded
    );
}
