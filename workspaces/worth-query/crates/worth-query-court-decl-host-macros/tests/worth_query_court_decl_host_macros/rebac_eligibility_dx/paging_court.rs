//! `#[worth_query(ability = ..)]` pages: a cursor carries a position, never
//! authority. Every page re-admits the reader's current ability, so revoking
//! it between page 1 and page 2 refuses page 2, through a session and through
//! a snapshot pinned before the revocation alike.
use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial;
use worth_query_host::facade::{WorthQueryRefusalCategory, WorthQueryUserFacingRefusal};

use super::program::{open, DOCUMENTS};
use super::*;

fn site_documents() -> SiteDocuments {
    SiteDocuments {
        site: SITE.to_owned(),
    }
}

fn two() -> NonZeroUsize {
    NonZeroUsize::new(2).unwrap()
}

fn assert_permission_denied(denial: WorthQueryApplicationRequestQueryDenial) {
    assert_eq!(
        denial.refusal_category(),
        WorthQueryRefusalCategory::PermissionDenied,
        "{denial:?}"
    );
}

#[test]
fn revoking_read_between_pages_refuses_the_next_page() {
    let path = temp_home_path("paging-session");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let carol = session(&home, &clock, "carol");
    let first = carol
        .query_page(site_documents(), two(), None)
        .expect("a reader pages the site");
    let ids: Vec<_> = first.rows().iter().map(|row| row.id.as_str()).collect();
    assert_eq!(ids, &DOCUMENTS[..2]);
    let cursor = first.next_cursor().expect("more documents remain").clone();
    // The same cursor still reads page 2 while carol reads the site.
    let second = carol
        .query_page(site_documents(), two(), Some(&cursor))
        .expect("page 2 while authorized");
    let ids: Vec<_> = second.rows().iter().map(|row| row.id.as_str()).collect();
    assert_eq!(ids, &DOCUMENTS[2..4]);

    revoke(&session(&home, &clock, "admin"), "carol-reads-site");
    assert_permission_denied(
        carol
            .query_page(site_documents(), two(), Some(&cursor))
            .expect_err("page 2 re-admits the revoked reader"),
    );
    assert_permission_denied(
        carol
            .query_page(site_documents(), two(), None)
            .expect_err("so does page 1"),
    );
}

#[test]
fn a_snapshot_pinned_before_revocation_still_re_admits_each_page() {
    let path = temp_home_path("paging-snapshot");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let carol = session(&home, &clock, "carol");
    let pinned = carol.snapshot().expect("carol pins the site");
    let first = pinned
        .query_page(site_documents(), two(), None)
        .expect("page 1 at the pin");
    let cursor = first.next_cursor().expect("more documents remain").clone();
    revoke(&session(&home, &clock, "admin"), "carol-reads-site");
    // The pin selects the data; authority is admitted now, at the head.
    assert_permission_denied(
        pinned
            .query_page(site_documents(), two(), Some(&cursor))
            .expect_err("a pinned page 2 re-admits the revoked reader"),
    );
}

#[test]
fn a_principal_without_the_ability_is_refused_the_first_page() {
    let path = temp_home_path("paging-stranger");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    assert_permission_denied(
        session(&home, &clock, "alice")
            .query_page(site_documents(), two(), None)
            .expect_err("alice holds no reader assignment"),
    );
}
