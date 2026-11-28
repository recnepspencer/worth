//! A multi-stage edit envelope shares one admission instant across every
//! stage and the commit, even when the source ticks on each read.
use crate::time::TickingClock;

use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome;

use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

use super::program::{open_with, owner};
use super::schema::StampLot;
use super::{at, date, lot_n};

#[test]
fn every_stage_of_one_envelope_and_its_commit_observe_one_ticking_reading() {
    let start = at(2026, 11, 27, 1, 30);
    let home = open_with(TickingClock::new(start));
    let session = home.session(owner(&home));
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        WorthQueryAdmissionClock::system(),
    );
    let request = session.request(&scope);
    let stamp = |lot| StampLot {
        lot,
        effective_on: date(2026, 11, 26),
        expires_on: date(2026, 12, 31),
    };
    let mut envelope = request
        .edit_envelope()
        .idempotency(11_u64)
        .runtime_stages(home.runtime())
        .ordered_authored()
        .expect("an ordered authored envelope");
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(stamp(1))
        .expect("stage one");
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(stamp(2))
        .expect("stage two");
    // Each stage handler reads twice; `TrustedDates` refuses any stamp that
    // differs from the commit's own instant.
    let outcome = envelope
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .expect("the envelope executes");
    assert!(
        matches!(outcome, WorthQueryEditEnvelopeOutcome::Committed(_)),
        "{outcome:?}"
    );
    let (first, second) = (lot_n(&home, 1).stamped_at, lot_n(&home, 2).stamped_at);
    assert_eq!(first, second, "both stages observe one instant");
    assert!(first.micros_since_unix_epoch >= start.micros_since_unix_epoch);
}
