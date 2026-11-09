//! Genuine recorded headers with one closed native corruption per court.
use super::*;

#[test]
fn cascade_refuses_an_empty_publication_roster() {
    check(
        false,
        "copied or mixed canonical authored cascade",
        Corruption::CascadeEmptyPublicationRoster,
    );
}

#[test]
fn cascade_refuses_a_repeated_authored_anchor() {
    check(
        false,
        "canonical cascade repeats or omits an authored anchor",
        Corruption::CascadeRepeatedAuthoredAnchor,
    );
}

#[test]
fn cascade_refuses_an_unowned_publication() {
    check(
        false,
        "canonical cascade contains an unowned or live publication",
        Corruption::CascadeUnownedPublication,
    );
}

#[test]
fn cascade_refuses_incomplete_origin_membership() {
    check(
        false,
        "copied or incomplete native output publication membership",
        Corruption::CascadeIncompleteOriginMembership,
    );
}

#[test]
fn cascade_refuses_a_repeated_route() {
    check(
        false,
        "canonical cascade repeats a route or member",
        Corruption::CascadeRepeatedRoute,
    );
}

#[test]
fn cascade_refuses_a_foreign_source() {
    check(
        false,
        "canonical cascade has a different source owner",
        Corruption::CascadeForeignSource,
    );
}

#[test]
fn cascade_refuses_a_cyclic_source() {
    check(
        false,
        "canonical cascade source ownership is cyclic",
        Corruption::CascadeCyclicSource,
    );
}
