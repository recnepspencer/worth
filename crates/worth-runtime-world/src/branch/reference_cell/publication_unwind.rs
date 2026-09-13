//! Thread-local unwind injection at actual materialization and movement boundaries.

use std::cell::Cell;

thread_local! {
    static ARMED: Cell<Option<Boundary>> = const { Cell::new(None) };
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Boundary {
    Materialized,
    Committed,
}

pub(crate) struct PublicationUnwind;

impl Drop for PublicationUnwind {
    fn drop(&mut self) {
        ARMED.set(None);
    }
}

pub(crate) fn arm() -> PublicationUnwind {
    arm_at(Boundary::Committed)
}

pub(crate) fn arm_materialized() -> PublicationUnwind {
    arm_at(Boundary::Materialized)
}

fn arm_at(boundary: Boundary) -> PublicationUnwind {
    assert!(
        ARMED.replace(Some(boundary)).is_none(),
        "one publication-boundary injection per thread"
    );
    PublicationUnwind
}

pub(super) fn after_committed() {
    reach(Boundary::Committed);
}

pub(super) fn after_materialized() {
    reach(Boundary::Materialized);
}

fn reach(boundary: Boundary) {
    if ARMED.get() == Some(boundary) {
        ARMED.set(None);
        panic!("injected unwind at the armed publication boundary");
    }
}
