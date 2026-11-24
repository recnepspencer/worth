//! Runtime census runner for the declared inline owner budgets.
pub(crate) use census::Census;
pub(crate) use worth_relational_contracts::internal_port::inline_type_sizes::census;

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
