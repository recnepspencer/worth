//! A multi-stage edit envelope shares one admission instant across every
//! stage and the commit, even when the source ticks on each read; undo and
//! redo commit at a trusted instant and are judged at it.
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome;
use worth_query_host::facade::primary_graph::{
    WorthQueryFixedRuntimeTimeSource, WorthQueryRuntimeTimeSource,
    WorthQueryRuntimeTimeSourceDenial,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

use super::invariant::EXPIRED;
use super::program::{open, open_with, owner};
use super::schema::StampLot;
use super::undo::{CorrectLotExpiry, ExtendLot};
use super::{at, date, lot, lot_n};

/// A trusted source that moves one millisecond forward on every read, so
/// any second read of the clock within one commit would be visible.
struct Ticking(Arc<AtomicI64>);

impl WorthQueryRuntimeTimeSource for Ticking {
    fn admission_clock(&self) -> WorthQueryAdmissionClock {
        WorthQueryAdmissionClock::system()
    }

    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        let micros = self.0.fetch_add(1_000, Ordering::SeqCst);
        u64::try_from(micros)
            .map(|micros| UNIX_EPOCH + Duration::from_micros(micros))
            .map_err(|_| WorthQueryRuntimeTimeSourceDenial::Unavailable)
    }
}

#[test]
fn every_stage_of_one_envelope_and_its_commit_observe_one_ticking_reading() {
    let start = at(2026, 11, 27, 1, 30).micros_since_unix_epoch;
    let home = open_with(Ticking(Arc::new(AtomicI64::new(start))));
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
    assert!(first.micros_since_unix_epoch >= start);
}

#[test]
fn undo_commits_at_a_trusted_instant_and_redo_is_judged_at_the_current_one() {
    let clock = WorthQueryFixedRuntimeTimeSource::new(at(2026, 11, 27, 1, 30));
    let home = open(Some(clock.clone()));
    let me = owner(&home);
    let session = home.session(&me);
    let extended = session
        .run(ExtendLot {
            lot: 1,
            expires_on: date(2027, 6, 30),
        })
        .expect("a future expiry commits");
    let entry = extended.correction_entry();

    // `TrustedDates` refuses a commit without trusted time, so a committed
    // undo proves the undo carried one.
    home.undo_authored::<CorrectLotExpiry>(&entry, &me)
        .expect("undo commits at a trusted instant");
    assert_eq!(lot(&home).expires_on, date(2999, 1, 1));

    // Redo re-applies the 30 June expiry at the current trusted instant.
    clock.set(at(2027, 7, 15, 12, 0));
    let declined = home
        .redo_authored::<CorrectLotExpiry>(&entry, &me)
        .expect("redo reaches a verdict");
    let refusal = declined
        .refusal_summary()
        .and_then(|summary| summary.domain_refusal().map(|r| r.code().to_owned()));
    assert_eq!(refusal.as_deref(), Some(EXPIRED), "{declined:?}");
    assert_eq!(lot(&home).expires_on, date(2999, 1, 1));
}
