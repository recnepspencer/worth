//! One edit envelope stamps one instant across a body birth, a handler and
//! the commit invariant, even under a source that ticks on every read; a
//! Directory Home reopens a birth with its original stamp.
use crate::time::TickingClock;
use crate::time::{controlled_at, unix_nanos};
use std::path::PathBuf;
use std::sync::Arc;

use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome;

use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

use super::program::{open, open_directory, owner};
use super::schema::{NoteDesk, OpenTicket};
use super::{at, desk, entry, ticket};

#[test]
fn a_body_birth_a_handler_and_the_commit_invariant_observe_one_ticking_instant() {
    let start = at(2026, 11, 27, 1, 30);
    let home = open(TickingClock::new(start));
    let session = home.session(owner(&home));
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        WorthQueryAdmissionClock::system(),
    );
    let mut envelope = session
        .request(&scope)
        .edit_envelope()
        .idempotency(21_u64)
        .runtime_stages(home.runtime())
        .ordered_authored()
        .expect("an ordered authored envelope");
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(OpenTicket { desk: 1, ticket: 5 })
        .expect("the body birth stage");
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(NoteDesk { desk: 1 })
        .expect("the handler stage");
    // `StampedAtAdmission` refuses a ticket or desk stamped at any instant
    // other than the commit's own, so a commit proves all three agree.
    let outcome = envelope
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .expect("the envelope executes");
    assert!(
        matches!(outcome, WorthQueryEditEnvelopeOutcome::Committed(_)),
        "{outcome:?}"
    );
    let born = ticket(&home, 5).created_at;
    assert_eq!(
        born,
        desk(&home).noted_at,
        "body and handler share one instant"
    );
    assert_eq!(
        born,
        entry(&home, 5).opened_at,
        "root and child share it too"
    );
    assert!(born.micros_since_unix_epoch >= start.micros_since_unix_epoch);
}

fn temp_home_path(label: &str) -> PathBuf {
    let mut path =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("body-admission-time-{label}"));
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    path
}

#[test]
fn a_directory_home_reopens_a_birth_with_its_original_stamp_after_the_clock_moves() {
    for clean in [true, false] {
        let path = temp_home_path(if clean { "clean" } else { "dropped" });
        let clock = controlled_at(at(2026, 11, 27, 1, 30));
        let home = open_directory(&path, clock.clone());
        home.session(owner(&home))
            .run(OpenTicket { desk: 1, ticket: 3 })
            .expect("the birth commits");
        if clean {
            home.close_clean().expect("Home closes");
        } else {
            drop(home);
        }
        clock.set_wall_unix_nanos(unix_nanos(at(2026, 12, 15, 12, 0)));
        let home = open_directory(&path, clock.clone());
        assert_eq!(ticket(&home, 3).created_at, at(2026, 11, 27, 1, 30));
        assert_eq!(entry(&home, 3).opened_at, at(2026, 11, 27, 1, 30));
        home.session(owner(&home))
            .run(OpenTicket { desk: 1, ticket: 4 })
            .expect("a birth after reopen commits");
        assert_eq!(ticket(&home, 4).created_at, at(2026, 12, 15, 12, 0));
        home.close_clean().expect("Home closes");
    }
}
