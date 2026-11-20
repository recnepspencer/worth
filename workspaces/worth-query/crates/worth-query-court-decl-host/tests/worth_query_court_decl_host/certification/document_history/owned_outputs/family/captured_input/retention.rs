//! A stored captured reader reached from its supplier's tree seals last (#254).
//!
//! Each world is born through an unchanged supplier edit that creates
//! `captured:capture:child` and captures its parents. The later edit changes
//! only model `carriage:d` through the same tree: `first:carriage:late` roots
//! it, its occurrence connections reach the stored reader, and its generator
//! connection owes `first:carriage:d`. The stored reader verifies
//! provisionally at its rank and seals once the routes that do not wait on it
//! settle: a changed read is then captured with normal admission and derived.
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome,
    runtime::ExecutionAllocationPolicy,
};

type Host = WorthQueryProgramApplicationRuntime<Schema, supplier::Program>;
const READER: &str = "captured:capture:child";

fn world(undeclared_anchor: bool) -> Host {
    let host: Host = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, captured_inputs: true, captured_undeclared_anchor: undeclared_anchor, ..Default::default() });
    if undeclared_anchor {
        // Store the owed child first: the reader does not capture it, so no
        // route through the supplier would order it after that child.
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let mut edit = (request.edit_envelope())
            .idempotency("owed-child-born".to_owned())
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<supplier::SupplierTree, _>(fixture::member::Intent {
            key: "capture:seed-a".into(),
            value: 1,
            key_derived_target: false,
        })
        .unwrap();
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    }
    // The seeded value: this edit only roots the supplier, reaching and
    // capturing the reader, whose parents are then all stored.
    let born = edit_supplier(&host, "supplier-born", 4);
    assert!(matches!(born, Outcome::Committed(_)), "{born:?}");
    host
}
pub(super) fn edit_supplier(host: &Host, key: &str, value: u64) -> Outcome {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = (request.edit_envelope())
        .idempotency(key.to_owned())
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<supplier::SupplierTree, _>(fixture::model::Intent {
        key: "carriage:d".into(),
        value,
        key_derived_target: false,
    })
    .unwrap();
    edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}
fn output(host: &Host, key: &str) -> Option<query::Row> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let rows = request
        .query(query::first_output::Read(key.into()))
        .execute()
        .ok()?;
    Some(rows.rows()[0].clone())
}
pub(super) fn row(host: &Host, key: &str) -> query::Row {
    output(host, key).unwrap_or_else(|| panic!("{key} exists"))
}

/// A known-answer court: the hand-computed sequential value, not an executed
/// sequential control. The declared captured parent `first:carriage:d` changes,
/// so the reader is derived from it after it settles.
#[test]
fn a_reader_reached_from_its_supplier_follows_its_owed_captured_parent() {
    let host = world(false);
    let reader = row(&host, READER);
    // Source 3 + seeds a10, late50 + 2 * anchor d40.
    assert_eq!(reader.value, 143);
    let late = row(&host, "first:carriage:late");
    let a = row(&host, "first:a");
    let outcome = edit_supplier(&host, "supplier-changed", 5);
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    let d = row(&host, "first:carriage:d");
    assert_eq!(d.value, 50, "the owed parent follows model carriage:d");
    let after = row(&host, READER);
    assert_eq!(
        after.value,
        3 + 10 + 50 + 2 * 50,
        "the reader followed its changed captured parent"
    );
    assert_eq!(after.native_id, reader.native_id);
    assert_eq!(row(&host, "first:carriage:late"), late);
    assert_eq!(row(&host, "first:a"), a);
}

/// A known-answer court for a read the capture does not declare: the reader
/// still seals last, so its re-read sees the settled child, and the edit
/// derives it instead of refusing.
#[test]
fn an_undeclared_changed_read_is_derived_from_the_settled_child() {
    let host = world(true);
    let reader = row(&host, READER);
    let owed = row(&host, "first:carriage:d");
    // Source 3 + seeds a10, late50 + the undeclared weight 9 * d40.
    assert_eq!((reader.value, owed.value), (3 + 10 + 50 + 9 * 40, 40));
    let outcome = edit_supplier(&host, "undeclared-changed", 5);
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(row(&host, "first:carriage:d").value, 50);
    let after = row(&host, READER);
    assert_eq!(after.value, 3 + 10 + 50 + 9 * 50, "derived from the settled child");
    assert_eq!(after.native_id, reader.native_id);
}

/// A healthy reader opens no capture, so no producer admission: an edit under
/// a principal admitted for the supplier needs nothing more for the reader.
#[test]
fn an_unchanged_supplier_retains_the_reader_without_derivation() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world(false);
    let keys = [READER, "first:a", "first:carriage:late", "first:carriage:d"];
    let before = keys.map(|key| row(&host, key));
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let captures = handler::captures_on_this_thread();
    let outcome = edit_supplier(&host, "supplier-unchanged", 4);
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(keys.map(|key| row(&host, key)), before);
    assert_eq!(
        handler::captures_on_this_thread(),
        captures,
        "a verified reader runs no capture"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "a verified reader and its unchanged parents derive nothing"
    );
}
