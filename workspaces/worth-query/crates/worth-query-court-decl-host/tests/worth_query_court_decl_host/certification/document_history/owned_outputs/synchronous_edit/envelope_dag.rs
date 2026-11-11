//! Declared dependencies order real D reads and refuse mixed cycles before effect.
use super::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;
fn host(worker: bool) -> WorthQueryProgramApplicationRuntime<Schema, fixture::Program> {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    world::install_configured(
        limits,
        2,
        None,
        world::Configuration {
            envelope_records: true,
            ..Default::default()
        },
    )
}
fn reordered(worker: bool) {
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host).0;
    let envelope = request
        .edit_envelope()
        .stage::<fixture::DependentTree, _>(fixture::dependent::Intent {
            key: "m".into(),
            value: 6,
            key_derived_target: false,
        })
        .stage::<fixture::ModelTree, _>(fixture::model::Intent {
            key: "a".into(),
            value: 3,
            key_derived_target: false,
        })
        .idempotency("reverse-dependency".to_owned());
    let dependent = envelope
        .stage_token::<fixture::dependent::Binding>(0)
        .unwrap();
    let model = envelope.stage_token::<fixture::model::Binding>(1).unwrap();
    let (outcome, writes) = observe_maintenance_work(8, || {
        envelope
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::Committed(receipt) = outcome else {
        panic!("a declared acyclic dependency refused")
    };
    assert_eq!(
        receipt.result(&dependent).unwrap().value,
        9,
        "the handler read the earlier unpublished Model value at D"
    );
    assert_eq!(
        receipt.result(&dependent).unwrap().ordinal,
        0,
        "DAG execution never changes canonical stage identity"
    );
    assert_eq!(receipt.result(&model).unwrap().ordinal, 1);
    assert_eq!(
        request
            .query(query::member_public::Read("m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        9
    );
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        3
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(courts::native_head(&host).1, [before]);
}
fn mixed_cycle(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let (outcome, writes) = observe_maintenance_work(8, || {
        request
            .edit_envelope()
            .stage::<fixture::CyclicTree, _>(fixture::cyclic::Intent {
                key: "a".into(),
                value: 3,
                key_derived_target: false,
            })
            .idempotency("mixed-cycle".to_owned())
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::CyclicStages(cycle) = outcome else {
        panic!("authored/producer cycle was not refused at admission")
    };
    assert_eq!(cycle.stages.len(), 1);
    assert_eq!(cycle.stages[0].binding, fixture::cyclic::Binding::IDENTITY);
    assert_eq!(cycle.stages[0].identity.ordinal(), 0);
    assert_eq!(cycle.overlaps.len(), 2);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), 0);
    assert_eq!(courts::native_head(&host), before);
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2
    );
}
fn lost_subject(worker: bool) {
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let fault = host
        .runtime()
        .break_next_coupled_assembly_without_a_subject_for_test();
    let (outcome, writes) = observe_maintenance_work(8, || {
        request
            .edit_envelope()
            .stage::<fixture::ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 3,
                key_derived_target: false,
            })
            .stage::<fixture::MemberTree, _>(fixture::member::Intent {
                key: "m".into(),
                value: 6,
                key_derived_target: false,
            })
            .idempotency("lost-envelope-subject".to_owned())
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    assert!(fault.taken_for_test());
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
        panic!("lost subject was not a custody refusal")
    };
    assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialKind::CoupledRefusalWithoutSubject);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(courts::native_head(&host), before);
}
#[test]
fn dependency_order_uses_the_candidate_and_preserves_ordinals() {
    reordered(false);
}
#[test]
fn dependency_order_uses_the_candidate_and_preserves_ordinals_worker() {
    reordered(true);
}
#[test]
fn authored_and_producer_cycle_refuses_before_publication() {
    mixed_cycle(false);
}
#[test]
fn authored_and_producer_cycle_refuses_before_publication_worker() {
    mixed_cycle(true);
}
#[test]
fn subjectless_envelope_break_is_a_custody_fault() {
    lost_subject(false);
}
#[test]
fn subjectless_envelope_break_is_a_custody_fault_worker() {
    lost_subject(true);
}
