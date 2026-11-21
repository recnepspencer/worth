//! A stored captured reader reached from its supplier's tree (#254 fixture).
//!
//! Each world is born through an unchanged supplier edit that creates
//! `captured:capture:child` and captures its parents. A healthy follow-up edit
//! retains the reader Verified with zero captures and zero producer contacts.
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};

type Host = WorthQueryProgramApplicationRuntime<Schema, supplier::Program>;
const READER: &str = "captured:capture:child";

fn world() -> Host {
    let host: Host = world::install_configured(
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        1,
        None,
        world::Configuration {
            envelope_records: true,
            interleaved_records: true,
            captured_inputs: true,
            captured_undeclared_anchor: false,
            ..Default::default()
        },
    );
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

/// A healthy reader opens no capture, so no producer admission: an edit under
/// a principal admitted for the supplier needs nothing more for the reader.
#[test]
fn an_unchanged_supplier_retains_the_reader_without_derivation() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world();
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
