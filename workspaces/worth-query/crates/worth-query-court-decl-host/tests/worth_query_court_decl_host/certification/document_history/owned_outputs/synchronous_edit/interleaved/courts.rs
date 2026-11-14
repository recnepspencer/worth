//! The ordinary public edit entry publishes one complete interleaved cabinet edit.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;
type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;
mod authored_only;
mod correction;
mod instance_replay;
mod positioned;
mod resolution;
fn host(
    second_derivation: Option<fn(u64) -> u64>,
    child_derivation: Option<fn(u64) -> u64>,
) -> Host {
    world::install_configured(host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, second_derivation, child_derivation, ..Default::default() })
}
// Declaration ordinals intentionally differ from DAG execution order.
macro_rules! edit {
    ($request:expr, $count:expr, $key:expr, $generation:expr, $frontier:expr) => {
        $request
            .edit_envelope()
            .stage::<ProducedTree, _>(dependent::Intent { add: 7 })
            .stage::<ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: $count,
                key_derived_target: false,
            })
            .stage::<ModelTree, _>(fixture::cap::Intent {
                key: "envelope-cap".into(),
                value: 8,
                key_derived_target: false,
            })
            .stage::<ApplicationNoOutputGraph, _>(seal::Intent {
                generation: $generation,
                frontier: $frontier,
                dependent: $count * 10 + 7,
            })
            .idempotency($key.to_owned())
    };
}
fn read_seal(host: &Host) -> reading::State {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.request(&principal, &scope)
        .query(reading::Read)
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
fn exercise(count: u64) {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let envelope = edit!(request, count, "one-interleaved-edit", 1, 4);
    let dependent = envelope.stage_token::<dependent::Binding>(0).unwrap();
    let model = envelope.stage_token::<fixture::model::Binding>(1).unwrap();
    let cap = envelope.stage_token::<fixture::cap::Binding>(2).unwrap();
    let seal = envelope.stage_token::<seal::Binding>(3).unwrap();
    let before = super::super::courts::native_head(&host).0;
    let (outcome, writes) = observe_maintenance_work(32, || {
        envelope
            .execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(receipt) = outcome else {
        panic!("interleaved edit refused: {outcome:?}")
    };
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(super::super::courts::native_head(&host).1, [before]);
    assert_eq!(receipt.stage_count(), 4);
    for (ordinal, result) in [
        (0, receipt.result(&dependent).unwrap()),
        (1, receipt.result(&model).unwrap()),
        (2, receipt.result(&cap).unwrap()),
    ] {
        assert_eq!(
            (result.ordinal, result.key.as_str()),
            (ordinal, "one-interleaved-edit")
        );
    }
    assert_eq!(
        receipt.result(&dependent).unwrap().value,
        count * 10 + 7,
        "B consumed A's new unpublished FirstOutput"
    );
    let result = receipt.result(&seal).unwrap();
    assert_eq!(
        (
            result.generation,
            result.frontier,
            result.dependent,
            result.first_child,
            result.ordinal,
            result.key.as_str()
        ),
        (2, 5, count * 10 + 7, 1001, 3, "one-interleaved-edit")
    );
    assert_eq!(
        receipt
            .stage_receipt(&dependent)
            .unwrap()
            .commit_reference(),
        receipt.receipt().commit_reference()
    );
    assert_eq!(
        receipt.stage_receipt(&seal).unwrap().commit_reference(),
        receipt.receipt().commit_reference()
    );
    let produced = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(produced.value, count * 10);
    assert!(
        receipt
            .receipt()
            .committed_changes()
            .entity_changes()
            .any(|(id, _)| id == produced.native_id),
        "B resolved an actual output allocated by this one commit"
    );
    for ordinal in 0..count {
        let child = request
            .query(query::second_output::Read(format!(
                "second:dependent:{ordinal}"
            )))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(child.value, 1001 + ordinal);
        assert!(receipt
            .receipt()
            .committed_changes()
            .entity_changes()
            .any(|(id, _)| id == child.native_id));
    }
    assert_eq!(
        read_seal(&host),
        reading::State {
            generation: 2,
            frontier: 5
        }
    );
    let head = super::super::courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (replayed, writes) = observe_maintenance_work(8, || {
        edit!(request, count, "one-interleaved-edit", 1, 4)
            .execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::RecoveredCommitted(replayed) = replayed else {
        panic!("replay fabricated a live result: {replayed:?}")
    };
    assert!(receipt.receipt().matches_recorded_original(&replayed));
    assert_eq!(
        replayed.commit_reference(),
        receipt.receipt().commit_reference()
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(super::super::courts::native_head(&host), head);
    assert_eq!(
        read_seal(&host),
        reading::State {
            generation: 2,
            frontier: 5
        }
    );
    assert!(matches!(
        edit!(request, count, "one-interleaved-edit", 2, 5)
            .execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::IdempotencyIntentDrift
    ));
    let (healthy, writes) = observe_maintenance_work(8, || {
        edit!(request, count, "healthy-interleaved-edit", 2, 5)
            .execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    assert!(matches!(healthy, Outcome::Committed(_)), "{healthy:?}");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "healthy roots and B's re-demand reuse actual current results"
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(
        read_seal(&host),
        reading::State {
            generation: 3,
            frontier: 6
        }
    );
}
#[test]
fn authored_and_derived_stages_share_one_commit_at_both_dependent_counts() {
    exercise(3);
    exercise(12);
}
fn refuses(
    second: Option<fn(u64) -> u64>,
    generation: u64,
    frontier: u64,
    expected_seal: Option<seal::Denial>,
    producer: &str,
) {
    let host = host(second, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::courts::native_head(&host);
    let envelope = edit!(request, 3, "refused-interleaved-edit", generation, frontier);
    let token = envelope.stage_token::<seal::Binding>(3).unwrap();
    let (outcome, writes) = observe_maintenance_work(16, || {
        envelope
            .execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    if let Some(expected) = expected_seal {
        let Outcome::DomainDenied(denial) = outcome else {
            panic!("seal did not own the refusal: {outcome:?}")
        };
        assert_eq!(denial.denial(&token).unwrap(), &expected);
        assert_eq!(denial.identity().ordinal(), 3);
    } else {
        let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
            panic!("producer refusal was lost: {outcome:?}")
        };
        assert!(
            denial
                .coupled_producer_denial()
                .unwrap()
                .subject()
                .contains(producer),
            "{denial:?}"
        );
    }
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(super::super::courts::native_head(&host), before);
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1
    );
    assert!(request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .is_err());
    assert!(request
        .query(query::second_output::Read("second:dependent:0".into()))
        .execute()
        .is_err());
    assert_eq!(
        read_seal(&host),
        reading::State {
            generation: 1,
            frontier: 4
        }
    );
}
#[test]
fn later_producer_and_seal_refusals_publish_no_prefix() {
    refuses(Some(|_| u64::MAX), 1, 4, None, "second");
    refuses(None, 2, 4, Some(seal::Denial::GenerationChanged), "");
    refuses(None, 1, 5, Some(seal::Denial::FrontierChanged), "");
}

#[test]
fn same_binding_disjoint_sources_with_a_real_dependent() {
    let host: Host = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        1,
        Some(2),
        world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let envelope = request
        .edit_envelope()
        .stage::<ProducedTree, _>(dependent::Intent { add: 7 })
        .stage::<ModelTree, _>(fixture::model::Intent {
            key: "a".into(),
            value: 3,
            key_derived_target: false,
        })
        .stage::<ModelTree, _>(fixture::model::Intent {
            key: "b".into(),
            value: 4,
            key_derived_target: false,
        })
        .idempotency("independent-sources-with-dependent".to_owned());
    let dependent = envelope.stage_token::<dependent::Binding>(0).unwrap();
    let a = envelope.stage_token::<fixture::model::Binding>(1).unwrap();
    let b = envelope.stage_token::<fixture::model::Binding>(2).unwrap();
    let before = super::super::courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, writes) = observe_maintenance_work(16, || {
        envelope.execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
    });
    let after = super::super::courts::native_head(&host);
    let publications = writes.published_rows.len() + writes.omitted_publications;
    eprintln!("instance-order entry: {outcome:#?}\npublications={publications}; head_before={before:?}; head_after={after:?}; producer_contacts={}",
        host.producer_contacts_on_this_thread_for_test() - contacts);
    let outcome = outcome.expect("the public entry executes the admitted request");
    let Outcome::Committed(receipt) = outcome else {
        assert_eq!(publications, 0);
        assert_eq!(after, before);
        panic!("independent sources plus their real dependent must commit: {outcome:#?}");
    };
    assert_eq!(publications, 1);
    assert_eq!(after.1, [before.0]);
    assert_eq!(receipt.stage_count(), 3);
    for (ordinal, result) in [
        (0, receipt.result(&dependent).unwrap()),
        (1, receipt.result(&a).unwrap()),
        (2, receipt.result(&b).unwrap()),
    ] {
        assert_eq!(
            (result.ordinal, result.key.as_str()),
            (ordinal, "independent-sources-with-dependent")
        );
    }
    assert_eq!(receipt.result(&dependent).unwrap().value, 37);
    assert_eq!(
        receipt
            .stage_receipt(&dependent)
            .unwrap()
            .commit_reference(),
        receipt.receipt().commit_reference()
    );
    for (key, value) in [("a", 3), ("b", 4)] {
        let model = request
            .query(query::model::Read(key.into()))
            .execute()
            .unwrap();
        assert_eq!(model.rows()[0].value, value);
        let output = request
            .query(query::first_output::Read(format!("first:{key}")))
            .execute()
            .unwrap();
        assert_eq!(output.rows()[0].value, value * 10);
    }
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let head = super::super::courts::native_head(&host);
    instance_replay::verify(&host, receipt.receipt());
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(super::super::courts::native_head(&host), head);
}
