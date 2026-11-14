//! Complete recursive context and per-capture static work share one supplier.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;
#[test]
fn recursive_component_retains_parents_and_later_static_selectors() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records:true, interleaved_records:true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::super::courts::native_head(&host).0;
    let mut edit = request
        .edit_envelope()
        .idempotency("component-and-static".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(fixture::model::Intent {
        key: "carriage:b".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    // This real read of first:a forces the earlier prerequisite wave before
    // its own unchanged-a root is appended, then supplies static selector a.
    edit.push_stage::<Tree, _>(fixture::after_producer::Intent {
        key: "a".into(),
        value: 1,
        key_derived_target: false,
    })
    .unwrap();
    edit.push_stage::<Tree, _>(fixture::member::Intent {
        key: "m".into(),
        value: 2,
        key_derived_target: false,
    })
    .unwrap();
    let ((outcome, contacts), writes) = observe_maintenance_work(24, || {
        let before = host.producer_contacts_on_this_thread_for_test();
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        (
            outcome,
            host.producer_contacts_on_this_thread_for_test() - before,
        )
    });
    let Outcome::Committed(receipt) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(receipt.stage_count(), 3);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(
        super::super::super::super::courts::native_head(&host).1,
        [before]
    );
    for (key, value) in [("a", 10), ("carriage:d", 40), ("carriage:b", 530)] {
        let rows = request
            .query(query::first_output::Read(format!("first:{key}")))
            .execute()
            .unwrap();
        assert_eq!(
            rows.rows()[0].value,
            value,
            "both typed prerequisite groups must execute before their consumer"
        );
    }
    for (key, value) in [("a", 1001), ("carriage:d", 1004)] {
        let rows = request
            .query(query::second_output::Read(format!("second:{key}")))
            .execute()
            .unwrap();
        assert_eq!(
            rows.rows()[0].value,
            value,
            "same-route aliases must keep each actual static selector"
        );
    }
    assert_eq!(
        contacts, 5,
        "three real first routes and two selected second routes, with no supplier replay"
    );
}
#[test]
fn fragmented_overlapping_components_refuse_at_declaration() {
    let schema = Schema::declaration().unwrap();
    let Err(cause) = ApplicationProgramAuthoring::<Schema, Fragmented>::begin()
        .with_schema(&schema)
        .validated_program()
    else {
        panic!("fragmented component admitted")
    };
    assert_eq!(
        cause.kind(),
        ApplicationProgramValidationDenialKind::IncompleteOccurrenceComponent
    );
}
