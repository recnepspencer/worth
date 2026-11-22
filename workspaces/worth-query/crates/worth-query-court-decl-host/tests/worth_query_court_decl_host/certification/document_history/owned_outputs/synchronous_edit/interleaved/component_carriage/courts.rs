//! Complete recursive context and per-capture static work share one supplier.
use super::*;
use crate::certification::document_history::owned_outputs::synchronous_edit::courts::native_head;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;
#[test]
fn a_later_authored_link_expands_the_same_atomic_output_closure() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records:true, interleaved_records:true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = native_head(&host).0;
    let mut edit = request
        .edit_envelope()
        .idempotency("new-dependent-after-parent".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(fixture::model::Intent {
        key: "a".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    let attached = edit
        .push_stage::<ApplicationNoOutputGraph, _>(attach::Intent {
            parent: "a".into(),
            child: "carriage:late".into(),
            pending_child: false,
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
    let Outcome::Committed(done) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        done.result(&attached).unwrap().value,
        30,
        "the later authored link reads the genuine earlier produced value"
    );
    assert_eq!(done.result(&attached).unwrap().ordinal, 1);
    assert_eq!(native_head(&host).1, [before]);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    let later = request
        .query(query::first_output::Read("first:carriage:late".into()))
        .execute()
        .unwrap();
    assert_eq!(
        later.rows()[0].value,
        (5 + 30) * 10,
        "only final outgoing discovery can require this newly linked consumer"
    );
    assert_eq!(
        contacts, 5,
        "a, b, d, the later consumer and a's static child each execute once"
    );
}
#[test]
fn recursive_component_retains_parents_and_later_static_selectors() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records:true, interleaved_records:true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = native_head(&host).0;
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
    edit.push_stage::<AfterTree, _>(fixture::after_producer::Intent {
        key: "envelope-cap".into(),
        value: 1,
        key_derived_target: false,
    })
    .unwrap();
    edit.push_stage::<MemberTree, _>(fixture::member::Intent {
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
    assert_eq!(native_head(&host).1, [before]);
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
    for (key, value) in [("a", 1000), ("carriage:d", 4000)] {
        let rows = request
            .query(query::child_output::Read(format!("from-first:first:{key}")))
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
        "three real first routes and two selected from-first routes, with no supplier replay"
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
    let Err(cause) = ApplicationProgramAuthoring::<Schema, DuplicateTail>::begin()
        .with_schema(&schema)
        .validated_program()
    else {
        panic!("duplicate inside one graph admitted")
    };
    assert_eq!(
        cause.kind(),
        ApplicationProgramValidationDenialKind::DuplicateConnection
    );
}

fn run(consumed: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records:true, interleaved_records:true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency(format!("pending-child:{consumed}"))
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(fixture::model::Intent {
        key: "a".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    let mut observed = None;
    if consumed {
        // This existing real stage needs First, exposing its pending static child.
        edit.push_stage::<AfterTree, _>(fixture::after_producer::Intent {
            key: "envelope-cap".into(),
            value: 1,
            key_derived_target: false,
        })
        .unwrap();
        // Admission stays on the existing authored Model; the actual child
        // value is read only after its producer barrier has run.
        observed = Some(
            edit.push_stage::<ApplicationNoOutputGraph, _>(value_observation::Intent {
                model: "a".into(),
                source: value_observation::ValueSource::Child("from-first:first:a".into()),
            })
            .unwrap(),
        );
    }
    let attached = edit
        .push_stage::<ApplicationNoOutputGraph, _>(attach::Intent {
            parent: "a".into(),
            child: "carriage:late".into(),
            pending_child: true,
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
    if consumed {
        // The late link changes the incoming selection of the consumed
        // child's parents (#238): the child selects again and derives again,
        // and the edit commits what one commit per stage commits.
        let Outcome::Committed(receipt) = outcome else {
            panic!("the reselected child commits the sequential world: {outcome:?}")
        };
        let link = receipt.result(&attached).unwrap();
        let (first_a, child) = consumed_child::outputs(&request);
        let coupled = consumed_child::World {
            first_a,
            child,
            observed: receipt.result(observed.as_ref().unwrap()).unwrap().value,
            attached: link.value,
        };
        // first:a = 3 * 10, its child 100 times that, observed as such.
        let expected = consumed_child::World {
            first_a: Some(30),
            child: Some(3000),
            observed: 3000,
            attached: 30,
        };
        assert_eq!(coupled, expected);
        assert_eq!(consumed_child::sequential(), expected);
        let _ = (writes, contacts, before);
    } else {
        let Outcome::Committed(receipt) = outcome else {
            panic!("{outcome:?}")
        };
        assert_eq!(receipt.result(&attached).unwrap().value, 30);
        assert_eq!(receipt.result(&attached).unwrap().ordinal, 1);
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
        assert_eq!(native_head(&host).1, [before.0]);
        assert_eq!(
            request
                .query(query::first_output::Read("first:carriage:late".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            50,
            "the late real prerequisite has no outgoing discovery substitute"
        );
        assert_eq!(
            request
                .query(query::child_output::Read("from-first:first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            3000
        );
        assert_eq!(
            contacts, 5,
            "four first routes and the root's selected child each execute once"
        );
    }
}
#[test]
fn a_pending_child_acquires_its_incoming_parents_when_needed() {
    run(false);
}
#[test]
fn a_consumed_child_selects_its_parents_again_and_commits_the_sequential_world() {
    run(true);
}

/// The observation stage's barrier prepares `first:a`; the later authored link
/// then changes the discovery that selected its incoming parents. Qualified
/// use reopens `first:a`, but its successor round's source reads the incoming
/// lineage edge its own chain linked: relations are outside the own-output
/// view (PR C §1.5, entities only), so the round stops before append with
/// #119 `PriorEffectRead` and the edit refuses atomically. The absent
/// supplier is never forced at the barrier.
#[test]
fn an_absent_supplier_does_not_force_its_prerequisite_at_an_authored_barrier() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records:true, interleaved_records:true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("absent-child-barrier".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(fixture::model::Intent {
        key: "a".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    // Child is an installed possible writer for this binding, but this actual
    // observation reads a Model and no Child occurrence has been requested.
    edit.push_stage::<ApplicationNoOutputGraph, _>(value_observation::Intent {
        model: "carriage:late".into(),
        source: value_observation::ValueSource::Model,
    })
    .unwrap();
    // This actual authored link changes First:a's incoming parent set.
    edit.push_stage::<ApplicationNoOutputGraph, _>(authored_parent::Intent {
        parent: "carriage:late".into(),
        child: "a".into(),
    })
    .unwrap();
    let (outcome, writes) = observe_maintenance_work(24, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        cause
            .authored_retirement_error()
            .expect("the successor round's Native refusal")
            .authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::CoupledContinuationProvenancePostponed {
            issue: 119,
            reason: worth_relational::facade::publication::CoupledProvenancePostponement::PriorEffectRead,
        }),
        "{cause:?}"
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(native_head(&host), before);
}

mod consumed_child;
mod sequential_availability;
