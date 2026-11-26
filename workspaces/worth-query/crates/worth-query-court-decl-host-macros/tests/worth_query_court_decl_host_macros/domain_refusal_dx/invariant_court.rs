//! A commit-boundary invariant's domain refusal crosses the session boundary
//! with its exact code and message, and the refused commit leaves the Home as
//! it was.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    primary_graph::{
        WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationUncommitted,
    },
    WorthQueryRefusalCategory as Category, WorthQuerySessionRunDenial, WorthQueryUserFacingRefusal,
};

use super::invariant::{capacity_message, BIN_CAPACITY, BIN_CAPACITY_EXCEEDED};
use super::program::{open, principal};
use super::schema::ReceiveLot;
use super::{assert_refusal, lot, temp_home_path};

#[test]
fn a_bin_capacity_violation_refuses_with_its_code_and_leaves_the_home_untouched() {
    let path = temp_home_path("invariant");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let session = home.session(principal(&home, &clock, "owner"));
    let before = home.committed_basis_version();

    let over = ReceiveLot {
        lot: 1,
        quantity: BIN_CAPACITY + 50,
    };
    let denial = session
        .run(over.clone())
        .expect_err("150 units overflow the bin");
    let WorthQuerySessionRunDenial::Uncommitted(WorthQueryApplicationUncommitted::Denied(commit)) =
        &denial
    else {
        panic!("the invariant refuses at the commit boundary: {denial:?}");
    };
    assert_eq!(commit.kind(), Kind::CustomInvariantDenied);
    let message = capacity_message(1, BIN_CAPACITY + 50);
    assert_refusal(denial.domain_refusal(), BIN_CAPACITY_EXCEEDED, &message);
    let summary = denial.user_summary();
    assert_eq!(summary.category(), Category::Validation);
    assert_eq!(summary.user_message(), message);
    assert_eq!(
        summary.domain_refusal().map(|r| r.code()),
        Some(BIN_CAPACITY_EXCEEDED)
    );

    // Atomic: no commit happened and the lot is unchanged.
    assert!(!home.committed_basis_version().changed_since(before));
    assert_eq!(lot(&home, &clock, 1).quantity, 40);

    // The same refusal through a spawned call on the owned session.
    let owned = session.clone().into_owned();
    let background = owned
        .spawn_run(over)
        .wait()
        .expect_err("spawned run refuses");
    assert_refusal(background.domain_refusal(), BIN_CAPACITY_EXCEEDED, &message);
    assert_eq!(lot(&home, &clock, 1).quantity, 40);

    // A quantity within capacity commits through the same invariant.
    session
        .run(ReceiveLot {
            lot: 1,
            quantity: BIN_CAPACITY,
        })
        .expect("a full bin commits");
    assert!(home.committed_basis_version().changed_since(before));
    assert_eq!(lot(&home, &clock, 1).quantity, BIN_CAPACITY);
    drop((owned, session));
    home.close_clean().expect("Home closes");
}
