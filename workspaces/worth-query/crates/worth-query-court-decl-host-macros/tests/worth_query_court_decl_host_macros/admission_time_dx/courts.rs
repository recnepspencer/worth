//! Anti-backdating, the Mountain day boundary, single-stamp consistency and
//! the system-clock default, each through a real Home commit.
use crate::time::controlled_at;
use std::time::Duration;
use worth_query_host::facade::runtime::time::{ControlledTime, system_time_sampler};

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationUncommitted,
};
use worth_query_host::facade::WorthQuerySessionRunDenial;

use super::invariant::{backdated_message, expired_message, mountain, BACKDATED, EXPIRED};
use super::program::{open, owner};
use super::schema::StampLot;
use super::{assert_refusal, at, date, lot};

/// 18:30 on 26 November in Mountain time, already 27 November in UTC.
fn thanksgiving_evening() -> std::sync::Arc<ControlledTime> {
    controlled_at(at(2026, 11, 27, 1, 30))
}

#[test]
fn a_forged_effective_date_is_refused_at_commit_and_leaves_the_home_untouched() {
    let clock = thanksgiving_evening();
    let home = open(Some(clock.clone()));
    let session = home.session(owner(&home));
    let before = home.committed_basis_version();
    let unstamped = lot(&home);

    let forged = StampLot {
        lot: 1,
        effective_on: date(2020, 1, 1),
        expires_on: date(2027, 1, 1),
    };
    let denial = session.run(forged).expect_err("a backdated lot is refused");
    let WorthQuerySessionRunDenial::Uncommitted(WorthQueryApplicationUncommitted::Denied(commit)) =
        &denial
    else {
        panic!("the invariant refuses at the commit boundary: {denial:?}");
    };
    assert_eq!(commit.kind(), Kind::CustomInvariantDenied);
    assert_refusal(denial.domain_refusal(), BACKDATED, &backdated_message(1));
    assert!(!home.committed_basis_version().changed_since(before));
    assert_eq!(lot(&home), unstamped);
}

#[test]
fn a_lot_is_valid_through_its_last_mountain_day_and_refused_at_mountain_midnight() {
    let clock = thanksgiving_evening();
    let home = open(Some(clock.clone()));
    let session = home.session(owner(&home));
    let last_day = date(2026, 11, 26);
    let stamp = |effective_on| StampLot {
        lot: 1,
        effective_on,
        expires_on: last_day,
    };

    // UTC has rolled over, Mountain has not: the 26th is still today.
    session.run(stamp(last_day)).expect("valid on its last day");
    let stamped = lot(&home);
    assert_eq!(
        (stamped.effective_on, stamped.expires_on),
        (last_day, last_day)
    );
    assert_eq!(stamped.stamped_at, at(2026, 11, 27, 1, 30));

    // 07:00Z is 00:00 on the 27th in Mountain time.
    clock.advance(Duration::from_secs(5 * 3_600 + 30 * 60));
    let denial = session
        .run(stamp(date(2026, 11, 27)))
        .expect_err("expired at Mountain midnight");
    assert_refusal(denial.domain_refusal(), EXPIRED, &expired_message(1));
    assert_eq!(lot(&home), stamped);
}

#[test]
fn the_handler_and_the_invariant_observe_one_instant_that_moves_only_with_the_clock() {
    let clock = thanksgiving_evening();
    let home = open(Some(clock.clone()));
    let session = home.session(owner(&home));
    let stamp = StampLot {
        lot: 1,
        effective_on: date(2026, 11, 27),
        expires_on: date(2026, 12, 31),
    };
    // The handler reads twice and the invariant refuses any stamp other than
    // its own reading, so a commit proves all three readings are one.
    session.run(stamp.clone()).expect("first stamp commits");
    assert_eq!(lot(&home).stamped_at, at(2026, 11, 27, 1, 30));
    clock.advance(Duration::from_secs(3_600));
    session.run(stamp).expect("second stamp commits");
    assert_eq!(lot(&home).stamped_at, at(2026, 11, 27, 2, 30));
}

fn sampled_wall_micros() -> i64 {
    (system_time_sampler().wall().unwrap().unix_nanos() / 1_000) as i64
}

#[test]
fn a_default_home_admits_with_the_live_system_clock() {
    let home = open(None);
    let session = home.session(owner(&home));
    let zone = mountain();
    // A run straddling Mountain midnight retries once with the new day.
    let committed = (0..2).find_map(|_| {
        let before = sampled_wall_micros();
        let today = zone.today().expect("today in Mountain time");
        let stamp = StampLot {
            lot: 1,
            effective_on: today,
            expires_on: date(2999, 12, 31),
        };
        let outcome = session.run(stamp);
        let after = sampled_wall_micros();
        outcome.ok().map(|_| (before, after))
    });
    let (before, after) = committed.expect("a default Home stamps live time");
    let stamped = lot(&home).stamped_at.micros_since_unix_epoch;
    assert!(stamped > 0);
    assert!(
        (before..=after).contains(&stamped),
        "{before} <= {stamped} <= {after}"
    );
}
