//! A constant-scope action always writes the main counter, and the invariant
//! declared inside the contribution refuses at the commit boundary with its
//! structured domain refusal, leaving the Home unchanged.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    primary_graph::{
        WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationUncommitted,
    },
    WorthQueryRefusalCategory as Category, WorthQuerySessionRunDenial, WorthQueryUserFacingRefusal,
};

use super::counter;
use super::invariant::{ceiling_message, CEILING, CEILING_EXCEEDED};
use super::program::{open, owner};
use super::schema::{ResetMain, MAIN_COUNTER};

#[test]
fn a_constant_scope_action_writes_its_fixed_counter_for_every_input() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&clock);
    let session = home.session(owner(&home, &clock));

    for value in [7, 640] {
        session
            .run(ResetMain { value })
            .expect("the main counter resets");
        assert_eq!(counter(&home, &clock, MAIN_COUNTER).value, value);
    }
    assert_eq!(
        counter(&home, &clock, 4).value,
        30,
        "no other counter moves"
    );
}

#[test]
fn a_contribution_declared_invariant_refuses_with_its_domain_reason_and_no_commit() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&clock);
    let session = home.session(owner(&home, &clock));
    let before = home.committed_basis_version();

    let over = CEILING + 1;
    let denial = session
        .run(ResetMain { value: over })
        .expect_err("the ceiling refuses the reset");
    let WorthQuerySessionRunDenial::Uncommitted(WorthQueryApplicationUncommitted::Denied(commit)) =
        &denial
    else {
        panic!("the invariant refuses at the commit boundary: {denial:?}");
    };
    assert_eq!(commit.kind(), Kind::CustomInvariantDenied);
    let message = ceiling_message(MAIN_COUNTER, over);
    let refusal = denial
        .domain_refusal()
        .expect("the run carries the invariant's domain refusal");
    assert_eq!(
        (refusal.code(), refusal.message()),
        (CEILING_EXCEEDED, message.as_str())
    );
    let summary = denial.user_summary();
    assert_eq!(summary.category(), Category::Validation);
    assert_eq!(summary.user_message(), message);

    assert!(!home.committed_basis_version().changed_since(before));
    assert_eq!(counter(&home, &clock, MAIN_COUNTER).value, 0);
    // At the ceiling itself the same action commits.
    session
        .run(ResetMain { value: CEILING })
        .expect("the ceiling admits its own value");
    assert_eq!(counter(&home, &clock, MAIN_COUNTER).value, CEILING);
}
