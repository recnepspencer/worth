//! Runtime World issues the Relational fork-source token itself. No caller
//! holds one, so the only thing standing between an admitted creation and a
//! branch forked from state nobody observed is the comparison the creation path
//! makes between the source it observes for itself and the source basis the
//! attempt was admitted against.

use super::*;

/// A creation that observes its own fork source and finds it moved must deny
/// before it reserves the destination, and the matching case must still fork.
#[test]
fn relational_fork_creation_compares_a_freshly_observed_source_token() {
    let (fixture, owner, source) = setup_with_relational_source(3);
    let history_before = owner.state.history.len();
    let matching = create_forked_branch(
        &owner,
        &source,
        fork_intent(
            "branch-source-token-match",
            relational_fork("relational-source-token-match"),
            SignalBranchCreationPlan::ReuseExact,
        ),
    );
    assert_new_branch_observation(&owner, &source, &matching, history_before);
    assert_eq!(owner.state.custody.installed(), 1);

    // The Relational owner advances its own source branch out of band. The
    // product head never moves, so nothing before the fork itself can notice.
    let _moved = fixture.perform_relational_owner_change();
    let lifecycles_before = super::super::owner_lifecycles(&owner);
    let branches_before = owner.state.branches.branch_count();
    let custody_before = owner.state.custody.installed();
    let cancellation = RuntimeWorldCancellationSource::new();
    let denial = RuntimeWorldBranchService::create_product_branch(
        &owner,
        RuntimeWorldBranchCreationRequest::new(
            source.clone(),
            fork_intent(
                "branch-source-token-stale",
                relational_fork("relational-source-token-stale"),
                SignalBranchCreationPlan::ReuseExact,
            ),
            &cancellation.token(),
        ),
    )
    .expect_err("a moved fork source cannot be forked as the admitted one");

    assert_eq!(denial, RuntimeWorldBranchAdmissionDenial::ForkSourceChanged);
    assert_eq!(
        super::super::owner_lifecycles(&owner),
        lifecycles_before,
        "a changed fork source denies before either component owner moves"
    );
    assert_eq!(owner.state.custody.installed(), custody_before);
    assert_eq!(owner.state.branches.branch_count(), branches_before);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert!(
        fixture
            .reserve_relational_fork_target("relational-source-token-stale")
            .is_ok(),
        "the denied destination was never created, so it is still reservable"
    );
}
