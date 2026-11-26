//! A hand-written handler's domain refusal keeps its exact code and message
//! through `run` and `spawn_run`, and an inadmissible reason is a defect.
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_entry::WorthQueryApplicationRequestMutationDenial as Request,
    primary_graph::{MutationHandlerExecutionDenial, WorthQueryOperationBodyDenial},
    WorthQueryBackgroundDenial, WorthQueryDomainRefusalDenial,
    WorthQueryRefusalCategory as Category, WorthQuerySessionRunDenial, WorthQueryUserFacingRefusal,
};

use super::handler::{expired_message, mislabeled_message, LOT_EXPIRED};
use super::program::{open, principal};
use super::schema::ShipLot;
use super::{assert_refusal, lot, temp_home_path, RunDenial};

fn ship(lot: u64, quantity: u64) -> ShipLot {
    ShipLot { lot, quantity }
}

#[test]
fn an_expired_lot_refuses_with_its_code_and_message_through_run_and_spawn_run() {
    let path = temp_home_path("handler");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let session = home.session(principal(&home, &clock, "owner"));

    let denial = session
        .run(ship(2, 1))
        .expect_err("an expired lot cannot ship");
    assert!(
        matches!(
            &denial,
            WorthQuerySessionRunDenial::DomainDenied(WorthQueryOperationBodyDenial::DomainRefused(
                _
            ))
        ),
        "{denial:?}"
    );
    assert_refusal(denial.domain_refusal(), LOT_EXPIRED, &expired_message(2));
    assert!(denial.keyed_entity().is_none());
    let summary = denial.user_summary();
    assert_eq!(summary.category(), Category::Validation);
    assert!(!summary.retryable());
    assert_eq!(summary.user_message(), expired_message(2));
    assert_eq!(summary.to_string(), expired_message(2));
    assert_eq!(
        summary.domain_refusal().map(|r| r.code()),
        Some(LOT_EXPIRED)
    );

    // The same refusal through the owned session and its call worker.
    let owned = session.clone().into_owned();
    let owned_denial = owned.run(ship(2, 1)).expect_err("owned run refuses");
    assert_refusal(
        owned_denial.domain_refusal(),
        LOT_EXPIRED,
        &expired_message(2),
    );
    let background = owned
        .spawn_run(ship(2, 1))
        .wait()
        .expect_err("spawned run refuses");
    assert!(
        matches!(background, WorthQueryBackgroundDenial::Denied(_)),
        "{background:?}"
    );
    assert_refusal(
        background.domain_refusal(),
        LOT_EXPIRED,
        &expired_message(2),
    );

    // A rule computed from state names itself too; nothing was committed.
    let short = session.run(ship(1, 41)).expect_err("41 of 40 units");
    assert_refusal(
        short.domain_refusal(),
        "insufficient_stock",
        "Only 40 units are on hand.",
    );
    assert_eq!(
        (
            lot(&home, &clock, 1).quantity,
            lot(&home, &clock, 2).quantity
        ),
        (40, 12)
    );

    // An unrefused shipment commits, and an ordinary refusal carries none.
    session.run(ship(1, 15)).expect("15 of 40 units ship");
    assert_eq!(lot(&home, &clock, 1).quantity, 25);
    drop((owned, session));
    home.close_clean().expect("Home closes");
}

fn defect(denial: &RunDenial) -> Option<WorthQueryDomainRefusalDenial> {
    let WorthQuerySessionRunDenial::Request(Request::Handler(
        MutationHandlerExecutionDenial::Handler(cause),
    )) = denial
    else {
        return None;
    };
    std::error::Error::source(cause)?
        .downcast_ref::<WorthQueryDomainRefusalDenial>()
        .copied()
}

#[test]
fn an_oversized_reason_is_a_deterministic_handler_defect_not_a_truncated_refusal() {
    let path = temp_home_path("handler-bounds");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let session = home.session(principal(&home, &clock, "owner"));
    let expected = WorthQueryDomainRefusalDenial::MessageTooLong {
        maximum: 256,
        observed: mislabeled_message().len(),
    };
    for _ in 0..2 {
        let denial = session
            .run(ship(3, 1))
            .expect_err("the handler's reason is too long");
        assert_eq!(defect(&denial), Some(expected), "{denial:?}");
        assert!(
            denial.domain_refusal().is_none(),
            "never a truncated reason"
        );
        assert_eq!(denial.refusal_category(), Category::Validation);
    }
    let owned = session.into_owned();
    let background = owned
        .spawn_run(ship(3, 1))
        .wait()
        .expect_err("spawned run refuses");
    assert!(background.domain_refusal().is_none());
    assert_eq!(lot(&home, &clock, 3).quantity, 5, "nothing was committed");
    drop(owned);
    home.close_clean().expect("Home closes");
}
