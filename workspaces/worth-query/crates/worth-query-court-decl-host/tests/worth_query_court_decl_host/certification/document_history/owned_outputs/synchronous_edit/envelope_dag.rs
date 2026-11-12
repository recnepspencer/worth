//! Declared dependencies order real D reads and refuse mixed cycles before effect.
use super::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::{
    application_entry::{
        DeclaredEditDependency, DeclaredEditParticipant as Participant,
        DeclaredEditReadScope as ReadScope, DeclaredEditScopeCycle, DeclaredEditStage,
        WorthQueryEditEnvelopeOutcome as Outcome,
    },
    domain::{
        WorthQueryOperationGraphReadScope as GraphRead, WorthQueryOperationTouchScope as Touch,
    },
    runtime::ExecutionAllocationPolicy,
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
    let DeclaredEditScopeCycle { stages, overlaps } = cycle;
    assert_eq!(stages.len(), 1);
    let DeclaredEditStage { binding, identity } = &stages[0];
    assert_eq!(*binding, fixture::cyclic::Binding::IDENTITY);
    assert_eq!(identity.ordinal(), 0);
    assert_eq!(overlaps.len(), 2);
    for dependency in overlaps {
        let entity = match dependency.writer {
            Participant::Stage(stage) => {
                assert_eq!(stage.binding, fixture::cyclic::Binding::IDENTITY);
                "Model"
            }
            Participant::Producer { family, .. } => {
                assert_eq!(family, first::Family::IDENTITY);
                "FirstOutput"
            }
        };
        let ReadScope::Graph(GraphRead::NativeProjection(read)) = dependency.read_scope else {
            panic!("the cycle must retain its declared field-read scope")
        };
        assert_eq!(read.entity().semantic_key(), entity);
    }
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
// This integration crate names every refusal type through the supported host facade.
fn assert_after_producer(dependency: &DeclaredEditDependency) {
    let Participant::Stage(stage) = &dependency.reader else {
        panic!("the refused reader must be the authored stage")
    };
    assert_eq!(stage.binding, fixture::after_producer::Binding::IDENTITY);
    assert_eq!(stage.identity.ordinal(), 0);
    let Participant::Producer {
        family,
        implementations,
    } = &dependency.writer
    else {
        panic!("the required writer must be the producer family")
    };
    assert_eq!(family, first::Family::IDENTITY);
    let mut implementations = implementations.clone();
    implementations.sort();
    assert_eq!(
        implementations,
        [
            "owned-outputs.first.initial",
            "owned-outputs.first.preserve"
        ]
    );
    let ReadScope::Graph(GraphRead::NativeProjection(read)) = &dependency.read_scope else {
        panic!("the overlap must name the declared FirstOutput field read")
    };
    assert_eq!(read.entity().semantic_key(), "FirstOutput");
    let paths = read
        .projection()
        .mask()
        .paths()
        .iter()
        .map(|path| {
            path.fields()
                .iter()
                .map(|field| field.as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(paths, [["FirstKey"], ["FirstValue"]]);
    let Touch::CreateEntity(write) = &dependency.write_scope else {
        panic!("creating FirstOutput overlaps its declared field read")
    };
    assert_eq!(write.entity(), "FirstOutput");
    assert_eq!(write.schema(), read.entity().schema());
}
fn stage_after_producer(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = host(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let (outcome, writes) = observe_maintenance_work(8, || {
        request
            .edit_envelope()
            .stage::<fixture::AfterProducerTree, _>(fixture::after_producer::Intent {
                key: "envelope-cap".into(),
                value: 3,
                key_derived_target: false,
            })
            .idempotency("stage-after-producer".to_owned())
            .execute_performed::<fixture::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
    });
    let Ok(Outcome::StageAfterProducer(dependency)) = outcome else {
        panic!("a stage requiring a producer output must refuse before execution")
    };
    assert_after_producer(&dependency);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), 0);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(courts::native_head(&host), before);
}
#[test]
fn stage_after_producer_names_the_declared_dependency_before_execution() {
    stage_after_producer(false);
}
#[test]
fn stage_after_producer_names_the_declared_dependency_before_execution_worker() {
    stage_after_producer(true);
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
