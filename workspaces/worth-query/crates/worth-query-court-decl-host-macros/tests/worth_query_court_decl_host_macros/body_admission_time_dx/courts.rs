//! A body birth stamps the commit's own trusted instant, deterministically,
//! and refuses without trusted time.
use std::time::Duration;

use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial as Request;
use worth_query_host::facade::primary_graph::{
    MutationHandlerExecutionDenial as Handler, WorthQueryFixedRuntimeTimeSource,
    WorthQueryRuntimeTimeDenial,
};
use worth_query_host::facade::WorthQuerySessionRunDenial;

use super::program::{open, owner};
use super::schema::OpenTicket;
use super::switchable::Switchable;
use super::{at, entry, ticket};

#[test]
fn a_body_birth_stamps_the_commit_instant_and_moves_only_with_the_clock() {
    let clock = WorthQueryFixedRuntimeTimeSource::new(at(2026, 11, 27, 1, 30));
    let home = open(clock.clone());
    let session = home.session(owner(&home));

    // `StampedAtAdmission` refuses any ticket whose `created_at` is not the
    // commit's `context.admitted_at()`, so this one commit proves the body's
    // stamp and the invariant's instant are the same reading.
    session
        .run(OpenTicket { desk: 1, ticket: 7 })
        .expect("the birth commits");
    assert_eq!(ticket(&home, 7).created_at, at(2026, 11, 27, 1, 30));
    // The child site is stamped in the same commit with the same instant.
    assert_eq!(entry(&home, 7).opened_at, at(2026, 11, 27, 1, 30));

    let later = clock
        .advance(Duration::from_secs(3_600))
        .expect("the advance is representable");
    session
        .run(OpenTicket { desk: 1, ticket: 8 })
        .expect("the second birth commits");
    assert_eq!(ticket(&home, 8).created_at, later);
    assert_eq!(entry(&home, 8).opened_at, later);
    assert_eq!(later, at(2026, 11, 27, 2, 30));
    assert_eq!(
        ticket(&home, 7).created_at,
        at(2026, 11, 27, 1, 30),
        "an earlier birth keeps its own instant"
    );
}

#[test]
fn an_unavailable_trusted_time_source_refuses_the_birth_and_writes_nothing() {
    let source = Switchable::new(WorthQueryFixedRuntimeTimeSource::new(at(
        2026, 11, 27, 1, 30,
    )));
    let home = open(source.clone());
    let session = home.session(owner(&home));
    let before = home.committed_basis_version();

    source.fail();
    let denial = session
        .run(OpenTicket { desk: 1, ticket: 9 })
        .expect_err("no birth without trusted time");
    // The body's own decision refused with the typed trusted-time denial; it
    // did not fall back to the host clock or stamp a fabricated instant.
    let WorthQuerySessionRunDenial::Request(Request::Handler(Handler::Handler(cause))) = denial
    else {
        panic!("the body decision refuses: {denial:?}");
    };
    assert_eq!(
        cause.downcast::<WorthQueryRuntimeTimeDenial>().ok(),
        Some(WorthQueryRuntimeTimeDenial::SourceUnavailable)
    );
    assert!(!home.committed_basis_version().changed_since(before));
}
