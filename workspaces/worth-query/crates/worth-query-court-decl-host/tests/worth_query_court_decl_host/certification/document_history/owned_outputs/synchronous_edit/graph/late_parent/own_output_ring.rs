//! A successor round that orbits or walks where its own chain changed the
//! graph (#277 step 4). Its view serves adjacency lists with the chain's own
//! relations as they were at the chain head. Native's walk and orbit
//! aggregate read the current image, and #277 does not serve them (public
//! #315). So a walk or orbit that crosses a list the chain relinked, reaches
//! a record the chain created or deleted, or starts at a member the chain
//! deleted never answers the round: it stops typed. A walk the chain did not
//! touch is the same in both images and answers.
use super::super::super::super::program;
use super::super::hierarchy;
use super::own_output_drop::first;
use super::*;
use family::c3b::ring_drop::{Edit, Probe, EDIT, PROBE, SEEN};
use family::c3b::ring_peer::{Peer, LINK};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
/// The child only writes member `m`.
mod writer {
    drop_program!(ring_drop, decision_write, "own-ring");
}
/// The child writes member `m` and can link `Beside` from the ring to a
/// record of its own, and on from there.
mod peer {
    drop_program!(ring_drop, c3b::ring_peer, "own-ring-peer");
}
const TYPED: &str =
    "CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }";
/// The next edit's ring edit, probe and peer link. Clears what the last saw.
fn stage(edit: Edit, probe: Probe, link: Peer) {
    EDIT.set(edit);
    PROBE.set(probe);
    LINK.set(link);
    SEEN.with_borrow_mut(Vec::clear);
}
/// The size of each probe that answered since the last `stage`.
fn seen() -> Vec<usize> {
    SEEN.with_borrow(Clone::clone)
}
/// One host on `$program`. Edit 1 (`a = 8`) builds the ring 0 → 1 → 2 → 0
/// under model `a` and commits `first:a` = 8 + 9. Edit 2 (`a = 9`) runs under
/// `$modes`: `ring-drop(a)` re-enters in its Preserve posture, its child sets
/// m = 10, and `first:a` reopens as a successor round of its own chain.
/// Binds edit 2's outcome text, its wave events, and whether the head moved.
macro_rules! second_edit {
    ($program:ident, $host:ident, $request:ident, $edit:ident, $modes:expr
     => $detail:ident, $events:ident, $moved:ident) => {
        drop_world!($program, execution, $host, $request, $edit);
        stage(Edit::Keep, Probe::None, Peer::Idle);
        let (outcome, events) = $edit("own-ring-first", 8);
        assert!(
            matches!(outcome, Outcome::Committed(_)),
            "the first edit commits the ring: {outcome:?}; {events:#?}"
        );
        assert_eq!(first(&$request), Some(8 + 9));
        let head = courts::native_head(&$host);
        let (ring, probe, link) = $modes;
        stage(ring, probe, link);
        let (outcome, $events) = $edit("own-ring-second", 9);
        let $detail = format!("{outcome:?}");
        let $moved = courts::native_head(&$host) != head;
    };
}
fn reopened(events: &[String]) -> bool {
    events.iter().any(|event| event == "reopen task=0 owed=0")
}

/// The head round drops member 1: the ring is 0 → 2 → 0 now, and 0 → 1 → 2 → 0
/// in the successor's view. The successor orbits member 0. Native's answer
/// (two members) crosses member 0's list, which holds the chain's unlink and
/// its link; it is not the orbit the round's own lists describe (three).
/// The round stops typed at that list, and only the head round's orbit
/// answered.
/// Red before step 4: the successor's orbit answered with the current ring.
#[test]
fn a_successor_orbiting_the_ring_its_chain_relinked_stops_typed_before_the_orbit_answers() {
    second_edit!(writer, host, request, edit, (Edit::Drop, Probe::Orbit(0), Peer::Idle)
        => detail, events, moved);
    assert!(
        detail.contains(TYPED),
        "{detail}; seen {:?}; {events:#?}",
        seen()
    );
    assert!(
        reopened(&events),
        "the refused round is the chain's successor: {events:#?}"
    );
    assert_eq!(seen(), [3], "only the head round's orbit answered");
    assert!(!moved);
    assert_eq!(first(&request), Some(8 + 9));
}

/// The same drop; the successor orbits member 1, which its view still holds
/// and its chain deleted. Native's aggregate read fails on the missing
/// member before any walk. The member is the chain's, so the stop is typed.
/// Red before step 4: untyped, nothing marked the view.
#[test]
fn a_successor_orbiting_the_member_its_chain_deleted_stops_typed() {
    second_edit!(writer, host, request, edit, (Edit::Drop, Probe::Orbit(1), Peer::Idle)
        => detail, events, moved);
    assert!(
        detail.contains(TYPED),
        "{detail}; seen {:?}; {events:#?}",
        seen()
    );
    assert!(
        reopened(&events),
        "the refused round is the chain's successor: {events:#?}"
    );
    assert_eq!(seen(), [3], "only the head round's orbit answered");
    assert!(!moved);
    assert_eq!(first(&request), Some(8 + 9));
}

/// The same drop; the successor walks forward from member 1. Native's walk
/// fails at its root, which is not live. The root is one the chain deleted,
/// so the stop is typed.
/// Red before step 4: untyped.
#[test]
fn a_successor_walking_from_the_member_its_chain_deleted_stops_typed() {
    second_edit!(writer, host, request, edit, (Edit::Drop, Probe::Walk(1), Peer::Idle)
        => detail, events, moved);
    assert!(
        detail.contains(TYPED),
        "{detail}; seen {:?}; {events:#?}",
        seen()
    );
    assert!(
        reopened(&events),
        "the refused round is the chain's successor: {events:#?}"
    );
    assert_eq!(seen(), [3], "only the head round's walk answered");
    assert!(!moved);
    assert_eq!(first(&request), Some(8 + 9));
}

/// The chain keeps its ring and only rewrites `first:a`. Behind its head
/// round, `ring-peer(a)` links `Beside` from member 0 to a record of its own.
/// The successor walks `Beside` from member 0. No relation of the chain's
/// lies on that walk, so it is the same in the successor's view and in the
/// current image: it answers with the peer's link, as a single round after
/// the peer would, and the edit commits.
/// Red before step 4: `PriorEffectRead`. A walk had no touch keys, so any
/// write of the chain's counted against it, and the peer's link made the head
/// and current walks differ.
#[test]
fn a_successor_walking_a_list_its_chain_did_not_touch_reads_the_current_walk_and_commits() {
    second_edit!(peer, host, request, edit, (Edit::Keep, Probe::Beside, Peer::Own)
        => detail, events, moved);
    assert!(
        detail.starts_with("Committed {"),
        "{detail}; seen {:?}; {events:#?}",
        seen()
    );
    assert!(
        reopened(&events),
        "the chain derived a successor round: {events:#?}"
    );
    assert_eq!(
        seen(),
        [1, 2],
        "the head round's walk, then the peer's link too"
    );
    assert!(moved);
    assert_eq!(first(&request), Some(9 + 10));
}

/// The head round inserts a new member between 2 and 0. Behind it,
/// `ring-peer(a)` links `Beside` from member 0 to a record of its own, and
/// from there to that new record. The successor walks `Beside` from member 0
/// and Native reaches the new record, which the successor's view does not
/// hold: its chain created it. The round stops typed there, and only the
/// head round's walk answered.
/// Red before step 4: the successor's walk answered with the created record.
#[test]
fn a_successor_whose_walk_reaches_a_record_its_chain_created_stops_typed() {
    second_edit!(peer, host, request, edit, (Edit::Grow, Probe::Beside, Peer::Onto("ring:a:3"))
        => detail, events, moved);
    assert!(
        detail.contains(TYPED),
        "{detail}; seen {:?}; {events:#?}",
        seen()
    );
    assert!(
        reopened(&events),
        "the refused round is the chain's successor: {events:#?}"
    );
    assert_eq!(seen(), [1], "only the head round's walk answered");
    assert!(!moved);
    assert_eq!(first(&request), Some(8 + 9));
}

/// A head round that only unlinks 0 → 1 leaves its orbit open, and Native
/// refuses that round itself. So no successor meets an orbit its chain
/// broke: the guard's arms for that break cannot be reached by an edit, and
/// are unit tests beside the guard (`native_read/walk/guard/tests.rs`).
#[test]
fn a_head_round_that_leaves_its_ring_open_is_refused_before_any_successor_runs() {
    second_edit!(writer, host, request, edit, (Edit::Open, Probe::Orbit(0), Peer::Idle)
        => detail, events, moved);
    assert!(
        detail.contains("Native aggregate identity preparation denied: InvalidPermutation"),
        "{detail}; {events:#?}"
    );
    assert!(!reopened(&events), "no successor round ran: {events:#?}");
    assert_eq!(seen(), [3], "only the head round's orbit answered");
    assert!(!moved);
    assert_eq!(first(&request), Some(8 + 9));
}

/// Edit 2 keeps the ring, and `ring-peer(a)` links `Beside` from member 0 to
/// a record of its own and on to member 1. It commits; the successor walked
/// all three. Edit 3's head round drops member 1. The peer's link to member 1
/// is another publication's, so the delete leaves it live. The successor
/// walks `Beside` from member 0 and Native fails at member 1: not a root, not
/// live, and still held by the successor's view. Native names the record,
/// the chain deleted it, and the stop is typed.
/// Red before Native named the record (`PinnedStructuralWalkDenial::endpoint`):
/// untyped `EndpointUnavailable`, nothing marked the view.
#[test]
fn a_successor_whose_walk_reaches_a_record_its_chain_deleted_stops_typed() {
    second_edit!(peer, host, request, edit, (Edit::Keep, Probe::Beside, Peer::Onto("ring:a:1"))
        => detail, events, moved);
    assert!(detail.starts_with("Committed {"), "{detail}; {events:#?}");
    assert_eq!(seen(), [1, 3], "the head round's walk, then both links");
    assert!(moved);
    let head = courts::native_head(&host);
    stage(Edit::Drop, Probe::Beside, Peer::Idle);
    let (outcome, events) = edit("own-ring-third", 10);
    let detail = format!("{outcome:?}");
    assert!(
        detail.contains(TYPED),
        "{detail}; seen {:?}; {events:#?}",
        seen()
    );
    assert!(
        reopened(&events),
        "the refused round is the chain's successor: {events:#?}"
    );
    assert_eq!(seen(), [3], "only the head round's walk answered");
    assert_eq!(courts::native_head(&host), head);
    assert_eq!(first(&request), Some(9 + 10));
}
