//! A required graph discovers children before its source becomes visible.
use super::super::hierarchy::program as hierarchy;
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
mod cleanup;
mod control;
mod decision_dependency;
mod diamond;
mod discovered;
mod envelope;
mod failure;
mod healthy;
mod independent;
mod late_parent;
mod many_routes;
mod middle_input;
mod partition;
mod permission;
mod program;
mod required_source_gate;
mod retention;
pub(super) mod unstable;

fn exercise(worker: bool, value: u64) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> =
        world::install_program_with_value(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let before = courts::native_head(&host).0;
    let performed = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value,
        })
        .without_source()
        .idempotency(&"required-graph".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let performed = match performed {
        WorthQueryApplicationPerformedMutationOutcome::Performed(done) => done,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(cause) => {
            panic!("source refused: {cause:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(cause) => {
            panic!("output retention refused: {:?}", cause.denial())
        }
        WorthQueryApplicationPerformedMutationOutcome::Blocked(_) => panic!("source blocked"),
        WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(_) => {
            panic!("source unpublished")
        }
        WorthQueryApplicationPerformedMutationOutcome::CleanupRequired { .. } => {
            panic!("source cleanup required")
        }
    };
    let source_receipt = performed.receipt().clone();
    let first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    if value == 1 {
        assert_eq!(first.value, 10);
        assert!(
            request
                .query(query::second_output::Read("second:a".into()))
                .execute()
                .is_err(),
            "discovery computed an empty dependent set"
        );
    } else {
        let second = request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!((first.value, second.value), (30, 1003));
    }
    let expected_contacts = contacts + if value == 1 { 1 } else { 2 };
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        expected_contacts
    );
    let parents = courts::native_head(&host).1;
    assert_eq!(
        parents,
        [before],
        "the source and both graph levels form one Native commit"
    );
    let (_, _, mut outputs) = performed
        .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
        .into_parts();
    let mut settled = false;

    for _ in 0..128 {
        if let WorthQueryApplicationProgramOutputProgress::Settled(done) =
            outputs.advance(&host, &request).unwrap()
        {
            assert!(
                done.root_receipt().is_none(),
                "Stable reuse carries no live receipt"
            );
            settled = true;
            break;
        }
    }
    assert!(settled, "current required outputs must settle");
    let origins = host.coupled_producer_origins_on_this_thread_for_test();
    assert_eq!(
        origins.len(),
        expected_contacts as usize - contacts as usize
    );
    for (producer, root, [current, described_current, before, same_commit]) in origins {
        eprintln!("actual Live origin runtime scoped to host, producer={producer}, root={root:?}");
        assert!(
            same_commit,
            "both describe the same commit, without the same origin posture"
        );
        assert!(
            current,
            "the performing member owns its actual current group origin"
        );
        assert!(
            !described_current,
            "a description alone cannot mint a group origin"
        );
        assert!(!before, "a real group origin cannot name another endpoint");
    }

    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        expected_contacts,
        "readiness reuses the actual co-committed outputs"
    );
    let replay = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value,
        })
        .without_source()
        .idempotency(&"required-graph".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::RecoveredCommitted(replayed) = replay else {
        panic!("the whole graph edit must replay its original receipt: {replay:?}")
    };
    assert!(source_receipt.matches_recorded_original(&replayed));
    assert_eq!(
        replayed.commit_reference(),
        source_receipt.commit_reference()
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        expected_contacts,
        "replay has no producer contacts"
    );
    let final_parents = courts::native_head(&host).1;
    assert_eq!(
        final_parents,
        [before],
        "finishing readiness contributes no Native commit"
    );
}
#[test]
fn required_graph_publishes_its_actual_discovered_child_in_the_source_commit() {
    exercise(false, 3)
}
#[test]
fn worker_only_required_graph_publishes_its_actual_discovered_child_in_the_source_commit() {
    exercise(true, 3)
}

#[test]
fn required_graph_computes_an_empty_dependent_set_from_its_actual_parent() {
    exercise(false, 1)
}
#[test]
fn worker_only_required_graph_computes_an_empty_dependent_set_from_its_actual_parent() {
    exercise(true, 1)
}
