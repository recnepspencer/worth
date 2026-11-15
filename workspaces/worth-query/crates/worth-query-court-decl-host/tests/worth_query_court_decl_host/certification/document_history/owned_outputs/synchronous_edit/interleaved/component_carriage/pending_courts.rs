//! A pending incoming read is delayed; a consumed incoming read remains strict.
use super::*;
use crate::certification::document_history::owned_outputs::synchronous_edit::courts::native_head;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;

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
    if consumed {
        edit.push_stage::<ApplicationNoOutputGraph, _>(pending_consumer::Intent {
            key: "from-first:first:a".into(),
        })
        .unwrap();
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
        let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
            panic!("{outcome:?}")
        };
        assert_eq!(
            cause.invariant_execution_failure().unwrap().kind(),
            WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
        );
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        assert_eq!(native_head(&host), before);
        assert!(request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .is_err());
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
            contacts, 6,
            "four first routes and two child routes each execute once"
        );
    }
}
#[test]
fn a_pending_child_acquires_its_incoming_parents_when_needed() {
    run(false);
}
#[test]
fn a_consumed_child_keeps_its_incoming_support_strict() {
    run(true);
}
