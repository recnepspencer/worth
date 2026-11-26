//! W8D: a multi-operation application without positional or nested
//! ceremony. Ten handler actions are configured by member name; three
//! features compose as one flat tuple; a fixed-duty action declares a
//! constant scope; an invariant is declared inside the contribution; a query
//! scoped on a mutable, non-unique field of its own root is refused at
//! declaration;
//! an ambiguous key says so instead of reporting a generic defect; and a
//! keyed refusal names an entity-prefixed field once.
mod adders;
mod builder_court;
mod duty_court;
mod invariant;
mod program;
mod schema;
mod scope_court;

use program::{owner, BoardHome};
use schema::{CounterView, ReadCounter};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;

/// The counter as committed, read back by the owner.
fn counter(home: &BoardHome, clock: &WorthQueryAdmissionClock, counter: u64) -> CounterView {
    let mut rows = home
        .session(owner(home, clock))
        .query(ReadCounter { counter })
        .expect("owner reads the counter");
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}
