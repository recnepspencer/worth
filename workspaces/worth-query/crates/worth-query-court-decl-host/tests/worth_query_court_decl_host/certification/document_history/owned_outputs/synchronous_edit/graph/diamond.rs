//! A converging child consumes both final parents exactly once in one edit.
use super::*;
use family::diamond as joined;
mod program;

fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let schema = Schema::declaration().unwrap();
    let host = in_memory_program(
        ApplicationProgramAuthoring::<Schema, program::Program>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        (world::Configuration::default(),),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        limits,
        |_, graph, installed| world::seed(graph, installed, 2, Some(7)),
    )
    .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut identity = None;
    for (value, expected_contacts) in [(3, 6), (4, 5)] {
        let before = courts::native_head(&host).0;
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let performed = request
            .advanced()
            .mutate(edit::Intent {
                key: "a".into(),
                value,
            })
            .without_source()
            .idempotency(&format!("diamond-{value}"))
            .execute_performed::<program::Program, program::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
            panic!("diamond edit refused")
        };
        let first = request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let left = request
            .query(query::child_output::Read("from-first:first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let right = request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let extra_a = request
            .query(query::member_public::Read("upstream:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let extra_b = request
            .query(query::member_public::Read("upstream:b".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!((extra_a.value, extra_b.value), (value * 5, 35));
        assert_ne!(
            extra_a.native_id, extra_b.native_id,
            "same family and partition retain distinct full-generation sources"
        );
        let output = request
            .query(query::member_public::Read("joined:first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(
            (first.value, left.value, right.value, output.value),
            (
                value * 10,
                value * 1000,
                value + 1000,
                value * 1000 + value + 1000
            )
        );
        if let Some(prior) = identity {
            assert_eq!(
                output.native_id, prior,
                "Preserve retains the shared child's identity"
            );
        }
        identity = Some(output.native_id);
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected_contacts,
            "the shared child runs once after both parents"
        );
        assert_eq!(
            courts::native_head(&host).1,
            [before],
            "source and all diamond writers publish once"
        );
        let (_, _, mut outputs) = performed
            .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
            .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
            .into_parts();
        let mut settled = false;
        for _ in 0..128 {
            if matches!(
                outputs.advance(&host, &request).unwrap(),
                WorthQueryApplicationProgramOutputProgress::Settled(_)
            ) {
                settled = true;
                break;
            }
        }
        assert!(settled, "all actual co-committed routes settle");
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected_contacts
        );
        assert_eq!(
            courts::native_head(&host).1,
            [before],
            "current diamond certification commits nothing"
        );
    }
}
#[test]
fn a_shared_child_consumes_both_final_parents_once_in_one_commit() {
    exercise(false)
}
#[test]
fn worker_only_a_shared_child_consumes_both_final_parents_once_in_one_commit() {
    exercise(true)
}

fn unequal(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host =
        world::install_program_with_other_model::<program::unequal::Program>(limits, 2, Some(7));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for (value, expected_contacts) in [(3, 7), (4, 5)] {
        let before = courts::native_head(&host).0;
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let outcome = request
            .advanced()
            .mutate(edit::Intent {
                key: "a".into(),
                value,
            })
            .without_source()
            .idempotency(&format!("unequal-diamond-{value}"))
            .execute_performed::<program::unequal::Program, program::unequal::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
            panic!("unequal diamond refused")
        };
        let middle = request
            .query(query::first_output::Read("first:b".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let extra_a = request
            .query(query::member_public::Read("upstream:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let extra_b = request
            .query(query::member_public::Read("upstream:b".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!((extra_a.value, extra_b.value), (value * 5, 35));
        assert_ne!(
            extra_a.native_id, extra_b.native_id,
            "same family and partition retain distinct full-generation sources"
        );
        let output = request
            .query(query::member_public::Read("joined:first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(middle.value, 70);
        assert_eq!(
            output.value,
            value * 1000 + value + 1000,
            "both unequal-depth parents precede their shared child"
        );
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected_contacts
        );
        assert_eq!(courts::native_head(&host).1, [before]);
        let (_, _, mut outputs) = performed
            .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
            .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
            .into_parts();
        let mut settled = false;
        for _ in 0..128 {
            if matches!(
                outputs.advance(&host, &request).unwrap(),
                WorthQueryApplicationProgramOutputProgress::Settled(_)
            ) {
                settled = true;
                break;
            }
        }
        assert!(settled);
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected_contacts
        );
        assert_eq!(courts::native_head(&host).1, [before]);
    }
}
#[test]
fn unequal_depth_parents_precede_their_shared_child() {
    unequal(false)
}
#[test]
fn worker_only_unequal_depth_parents_precede_their_shared_child() {
    unequal(true)
}
