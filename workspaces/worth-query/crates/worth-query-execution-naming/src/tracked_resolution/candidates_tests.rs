//! The owner-restriction rule over matched candidates: a pure function of the
//! matched records, the selection's arity and each record's owners.
use std::cell::Cell;

use worth_query_declaration::facade::application_naming::{
    NameCandidate, NameResolutionDenial, ResolvedName, WorthQueryName,
};

use super::NameResolutionCandidates;
use crate::admission::test_name;

struct Face;
type Candidates = NameResolutionCandidates<Face, &'static str>;
type Settled = Result<ResolvedName<&'static str>, NameResolutionDenial<Face, &'static str>>;

fn name(byte: u8) -> WorthQueryName<Face> {
    test_name(byte)
}

/// Two pieces: `left` owned by `a`, `right` owned by `b`.
fn pieces(selects_all: bool) -> Candidates {
    Candidates::matched(vec![(name(1), "left"), (name(2), "right")], selects_all)
}

fn owners(handle: &&'static str) -> Result<Vec<&'static str>, ()> {
    Ok(match *handle {
        "left" => vec!["a"],
        "right" => vec!["b"],
        "shared" => vec!["a", "b"],
        _ => Vec::new(),
    })
}

fn candidate(byte: u8, held_by: &[&'static str]) -> NameCandidate<Face, &'static str> {
    NameCandidate::new(name(byte), held_by.to_vec())
}

fn within(candidates: Candidates, owner: &'static str) -> Settled {
    candidates.resolve_in(&owner, owners).unwrap()
}

#[test]
fn one_match_is_exact_everywhere_and_reads_no_owner_unscoped() {
    let reads = Cell::new(0);
    let counted = |handle: &&'static str| {
        reads.set(reads.get() + 1);
        owners(handle)
    };
    let one = || Candidates::matched(vec![(name(1), "left")], false);
    assert_eq!(one().resolve(counted), Ok(Ok(ResolvedName::Exact("left"))));
    assert_eq!(reads.get(), 0, "an exact answer needs no owner");
    assert_eq!(within(one(), "a"), Ok(ResolvedName::Exact("left")));
    assert!(!one().is_ambiguous());
}

#[test]
fn several_matches_across_owners_refuse_ambiguous_with_each_owner() {
    assert!(pieces(false).is_ambiguous());
    assert_eq!(
        pieces(false).resolve(owners),
        Ok(Err(NameResolutionDenial::Ambiguous {
            candidates: vec![candidate(1, &["a"]), candidate(2, &["b"])],
        }))
    );
    // `AllDescendants` answers them all and reads no owner.
    let unread = |_: &&'static str| -> Result<Vec<&'static str>, ()> { Err(()) };
    assert_eq!(
        pieces(true).resolve(unread),
        Ok(Ok(ResolvedName::Plural(vec!["left", "right"])))
    );
}

#[test]
fn an_owner_scope_keeps_only_that_owners_records() {
    for selects_all in [false, true] {
        assert_eq!(
            within(pieces(selects_all), "a"),
            Ok(ResolvedName::Exact("left"))
        );
        assert_eq!(
            within(pieces(selects_all), "b"),
            Ok(ResolvedName::Exact("right"))
        );
    }
}

#[test]
fn an_owner_holding_no_match_refuses_and_never_falls_back() {
    for selects_all in [false, true] {
        assert_eq!(
            within(pieces(selects_all), "c"),
            Err(NameResolutionDenial::NoCandidateInOwner {
                candidates: vec![candidate(1, &["a"]), candidate(2, &["b"])],
            })
        );
    }
    // Even a single live match elsewhere is not an answer for this owner.
    let one = Candidates::matched(vec![(name(1), "left")], false);
    assert_eq!(
        within(one, "b"),
        Err(NameResolutionDenial::NoCandidateInOwner {
            candidates: vec![candidate(1, &["a"])],
        })
    );
}

#[test]
fn several_matches_within_one_owner_are_plural_or_ambiguous_by_selection() {
    let both = |selects_all| {
        Candidates::matched(
            vec![(name(1), "left"), (name(2), "right"), (name(3), "shared")],
            selects_all,
        )
    };
    assert_eq!(
        within(both(true), "a"),
        Ok(ResolvedName::Plural(vec!["left", "shared"]))
    );
    // Only the scoped owner's candidates are reported, each with its owners.
    assert_eq!(
        within(both(false), "a"),
        Err(NameResolutionDenial::Ambiguous {
            candidates: vec![candidate(1, &["a"]), candidate(3, &["a", "b"])],
        })
    );
}

#[test]
fn retired_and_refused_names_settle_without_owner_reads() {
    let unread = |_: &&'static str| -> Result<Vec<&'static str>, ()> { Err(()) };
    let retired = || Candidates::retired(9);
    assert_eq!(
        retired().resolve(unread),
        Ok(Ok(ResolvedName::Retired { at_commit: 9 }))
    );
    assert_eq!(
        retired().resolve_in(&"a", unread),
        Ok(Ok(ResolvedName::Retired { at_commit: 9 }))
    );
    let unknown = || Candidates::refused(NameResolutionDenial::UnknownName);
    assert_eq!(
        unknown().resolve_in(&"a", unread),
        Ok(Err(NameResolutionDenial::UnknownName))
    );
    assert_eq!(unknown().handles().count(), 0);
}

#[test]
fn a_failed_owner_read_fails_the_resolution_instead_of_guessing() {
    let failing = |_: &&'static str| -> Result<Vec<&'static str>, &'static str> { Err("hop") };
    assert_eq!(pieces(false).resolve_in(&"a", failing).err(), Some("hop"));
    assert_eq!(pieces(false).resolve(failing).err(), Some("hop"));
}
