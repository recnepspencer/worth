//! An internal trigger is a real effect even when application rows are equal.
use super::*;
use worth_relational::facade::inspection::observe_maintenance_work;
worth_query_effect!(pub Changed for Schema, payload query::RowBinding);
fn run(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_configured::<program::Lazy>(
        limits,
        2,
        None,
        world::Configuration {
            reconciliation_emission: true,
            ..world::Configuration::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let _initial = courts::settle(&host, &request);
    adoption::raise_cap(&host, &request);
    let _preserve = courts::settle(&host, &request);
    let expected = request
        .query(snapshot::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let changed = request
        .advanced()
        .mutate(member_edit::Intent {
            key: "m".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"emit-on-equal-output".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        changed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let (_, effect) = observe_maintenance_work(8, || {
        courts::settle_with_result(&host, &request, |settled| {
            assert!(
                settled.root_receipt().is_some(),
                "a trigger-bearing completion has a real receipt"
            );
            assert_eq!(settled.root_receipt().unwrap().emitted_effect_count(), 1);
        })
    });
    assert_eq!(
        effect.published_rows.len() + effect.omitted_publications,
        1,
        "an emitted trigger requires a real publication even with identity-equal application rows"
    );
    assert_eq!(
        request
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0],
        expected
    );
}
#[test]
fn an_equal_output_with_an_internal_trigger_publishes() {
    run(false);
}
#[test]
fn worker_only_equal_output_with_an_internal_trigger_publishes() {
    run(true);
}
