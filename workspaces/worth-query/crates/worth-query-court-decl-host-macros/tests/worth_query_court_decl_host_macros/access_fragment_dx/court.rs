//! Installed actions whose access comes from included fragments commit what
//! they declare and are refused what they do not.
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;

use super::moves::{Adjust, READ_LABEL, WRITE_LABEL};
use super::program::{open, owner, StockHome};
use super::receipts::Receive;
use super::schema::{BinView, MovementView, ReadBin, ReadMovement};
use super::transfers::Ship;

fn bin(home: &StockHome, clock: &WorthQueryAdmissionClock, bin: u64) -> BinView {
    let mut rows = home
        .session(owner(home, clock))
        .query(ReadBin { bin })
        .expect("owner reads the bin");
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}

fn movements(
    home: &StockHome,
    clock: &WorthQueryAdmissionClock,
    movement: u64,
) -> Vec<MovementView> {
    home.session(owner(home, clock))
        .query(ReadMovement { movement })
        .expect("owner reads movements")
}

#[test]
fn a_fragment_only_action_commits_its_posting_and_is_refused_the_undeclared_label() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&clock);
    let session = home.session(owner(&home, &clock));

    session
        .run(Adjust {
            bin: 1,
            add: 3,
            probe: 0,
        })
        .unwrap_or_else(|denial| panic!("the included posting access admits: {denial:?}"));
    assert_eq!(bin(&home, &clock, 1).on_hand, 13);

    // The handler's own refusal names the undeclared read or effect.
    for (probe, undeclared) in [
        (READ_LABEL, "UndeclaredDecisionTarget"),
        (WRITE_LABEL, "UndeclaredEffect"),
    ] {
        let refused = session.run(Adjust {
            bin: 1,
            add: 3,
            probe,
        });
        let Err(denial) = refused else {
            panic!("probe {probe}: the fragment names no label access");
        };
        assert!(format!("{denial:?}").contains(undeclared), "{denial:?}");
    }
    let unchanged = bin(&home, &clock, 1);
    assert_eq!(
        (unchanged.on_hand, unchanged.label.as_str()),
        (13, "seeded")
    );
}

#[test]
fn a_fragment_with_inline_access_books_a_lot_and_records_its_linked_movement() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&clock);
    let session = home.session(owner(&home, &clock));

    session
        .run(Receive {
            bin: 2,
            lot: 7,
            movement: 100,
        })
        .unwrap_or_else(|denial| panic!("included and inline access admit: {denial:?}"));
    assert_eq!(bin(&home, &clock, 2).on_hand, 5);
    assert_eq!(
        movements(&home, &clock, 100),
        [MovementView { id: 100, moved: 5 }]
    );
}

#[test]
fn two_overlapping_fragments_compose_into_one_installed_access_set() {
    let clock = WorthQueryAdmissionClock::system();
    // Both fragments read and write `on_hand`; the Home opening at all
    // proves the repeat registered once rather than as a duplicate member.
    let home = open(&clock);
    let session = home.session(owner(&home, &clock));

    session
        .run(Ship {
            bin: 1,
            quantity: 4,
            movement: 200,
        })
        .unwrap_or_else(|denial| panic!("composed fragments admit: {denial:?}"));
    assert_eq!(bin(&home, &clock, 1).on_hand, 6);
    assert_eq!(
        movements(&home, &clock, 200),
        [MovementView { id: 200, moved: 4 }]
    );
    assert_eq!(bin(&home, &clock, 2).on_hand, 0);
}
