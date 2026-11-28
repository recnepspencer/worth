//! W8F: one trusted admission instant per operation. A handler reads it from
//! `DecisionReader::admitted_at()`, a commit invariant from
//! `WorthQueryApplicationInvariantContext::admitted_at()`, and both see the
//! same reading of the Home's runtime time source; a held
//! `std::sync::Arc<ControlledTime>` makes them deterministic, while the
//! default Home reads the system clock. Caller-forged dates are refused at
//! commit, judged in the zone the application chooses.
mod courts;
mod envelope_clock;
mod handler;
mod invariant;
mod program;
mod recovery_court;
mod schema;

use worth_query_host::facade::runtime::time::ControlledTime;
use program::{owner, DatedHome};
use schema::{LotView, ReadLot};
use worth_query_decl::facade::application_schema::{CanonicalDate, CanonicalTimestamp};
use worth_query_host::facade::WorthQueryDomainRefusal;

/// Lot 1 as committed, read back by the owner.
fn lot(home: &DatedHome) -> LotView {
    lot_n(home, 1)
}

/// A lot as committed, read back by the owner.
fn lot_n(home: &DatedHome, lot: u64) -> LotView {
    let mut rows = home
        .session(owner(home))
        .query(ReadLot { lot })
        .expect("owner reads the lot");
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}

fn at(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> CanonicalTimestamp {
    CanonicalTimestamp::from_ymd_hms_micros(year, month, day, hour, minute, 0, 0).unwrap()
}

fn date(year: i32, month: u8, day: u8) -> CanonicalDate {
    CanonicalDate::from_ymd(year, month, day).unwrap()
}

fn assert_refusal(refusal: Option<WorthQueryDomainRefusal>, code: &str, message: &str) {
    let refusal = refusal.expect("the run carries a domain refusal");
    assert_eq!((refusal.code(), refusal.message()), (code, message));
}
