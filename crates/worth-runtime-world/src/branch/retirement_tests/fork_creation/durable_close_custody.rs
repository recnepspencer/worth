//! The durable close predicate: an installed record resolves only when its sibling
//! incarnation is an installed (published) branch; a reservation never resolves.

use super::*;

use crate::branch::registry::ProductBranchRegistry;
use crate::branch::CustodyComponent;

#[test]
fn durable_close_custody_resolves_only_records_of_installed_siblings() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let forked = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "durable-close-sibling",
            relational_fork("durable-close-relational"),
            signal_fork("durable-close-signal"),
        ),
    );
    assert_eq!(owner.state.custody.installed(), 2);
    let custody = &owner.state.custody;
    assert_eq!(
        custody.try_unresolved_close_custody(false, &owner.state.branches),
        Ok(true),
        "a memory-only owner keeps every installed record as custody"
    );
    assert_eq!(
        custody.try_unresolved_close_custody(true, &owner.state.branches),
        Ok(false),
        "a durable Home's records of an installed sibling are resolved"
    );
    let construction =
        crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract::new().unwrap();
    let without_sibling = ProductBranchRegistry::new(
        construction.owner_identity(),
        crate::budget::RuntimeWorldBudgets::without_policy_limits().live_product_branches(),
    );
    assert_eq!(
        custody.try_unresolved_close_custody(true, &without_sibling),
        Ok(true),
        "the same records without their sibling's installed cell stay unresolved"
    );
    let reservation = custody.reserve(CustodyComponent::Signal).unwrap();
    assert_eq!(
        custody.try_unresolved_close_custody(true, &owner.state.branches),
        Ok(true),
        "a reservation is unresolved even when every installed sibling is published"
    );
    assert_eq!(custody.try_unresolved_closed_custody(true), Ok(true));
    drop(reservation);
    assert_eq!(custody.try_unresolved_closed_custody(true), Ok(false));
    drop(forked);
}
