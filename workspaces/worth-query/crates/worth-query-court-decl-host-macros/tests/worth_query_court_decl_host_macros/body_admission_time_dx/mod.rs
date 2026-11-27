//! W9B: a declarative creation body stamps a required `CanonicalTimestamp`
//! from trusted admission time with
//! `creation.initialize_from_admission_time(field)`. The stamp is the one
//! instant the advancement settled: the handler's `DecisionReader::admitted_at()`
//! and the commit invariant's `context.admitted_at()` observe the same value,
//! it moves only with the trusted clock, survives Directory reopen, and an
//! unavailable time source refuses the birth instead of fabricating time.
mod courts;
mod envelope_and_reopen_court;
mod handler;
mod invariant;
mod program;
mod schema;
mod switchable;

use program::{owner, DeskHome};
use schema::{DeskView, ReadDesk, ReadTicket, TicketView};
use worth_query_decl::facade::application_schema::CanonicalTimestamp;

/// A ticket as committed, read back by the owner.
fn ticket(home: &DeskHome, ticket: u64) -> TicketView {
    let mut rows = home
        .session(owner(home))
        .query(ReadTicket { ticket })
        .expect("owner reads the ticket");
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}

/// The desk as committed, read back by the owner.
fn desk(home: &DeskHome) -> DeskView {
    let mut rows = home
        .session(owner(home))
        .query(ReadDesk { desk: 1 })
        .expect("owner reads the desk");
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}

fn at(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> CanonicalTimestamp {
    CanonicalTimestamp::from_ymd_hms_micros(year, month, day, hour, minute, 0, 0).unwrap()
}
