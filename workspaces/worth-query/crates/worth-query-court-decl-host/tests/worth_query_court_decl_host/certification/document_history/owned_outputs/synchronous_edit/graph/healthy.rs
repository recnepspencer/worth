//! Healthy parents compute discovery without opening a producer or writer.
use super::*;
mod program;
fn exercise<const CHILD_B: bool>(worker: bool, cold: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let mut host =
        world::install_program_with_other_model::<program::Program<CHILD_B>>(limits, 2, Some(5));
    let scope = request_scope();
    let mut original = None;
    for (key, value, expected_contacts) in [("a", 3, 2), ("b", 7, usize::from(CHILD_B))] {
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let before = courts::native_head(&host).0;
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let outcome = request
            .advanced()
            .mutate(edit::Intent {
                key: key.into(),
                value,
            })
            .without_source()
            .idempotency(&format!("healthy-parent-{key}"))
            .execute_performed::<program::Program<CHILD_B>, program::Tree<CHILD_B>>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = outcome else {
            panic!("healthy parent edit must perform")
        };
        let first = request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let second = request
            .query(query::second_output::Read(
                if CHILD_B { "second:b" } else { "second:a" }.into(),
            ))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(first.value, 30);
        assert_eq!(
            second.value,
            if CHILD_B {
                1000 + if key == "a" { 5 } else { 7 }
            } else {
                1003
            }
        );
        if let Some((first_id, second_id)) = original {
            assert_eq!((first.native_id, second.native_id), (first_id, second_id));
        } else {
            original = Some((first.native_id, second.native_id));
        }
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected_contacts as u64,
            "only the actually stale child may contact its producer"
        );
        assert_eq!(courts::native_head(&host).1, [before]);
        let head = courts::native_head(&host);
        let (_, _, mut outputs) = done
            .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
            .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
            .into_parts();
        let mut settled = false;
        for _ in 0..128 {
            if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
                outputs.advance(&host, &request).unwrap()
            {
                settled = true;
                break;
            }
        }
        assert!(settled);
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected_contacts as u64
        );
        assert_eq!(
            courts::native_head(&host),
            head,
            "readiness is contact and commit free"
        );
        if cold && key == "a" {
            drop(outputs);
            let checkpoint = super::super::failure::checkpoint(&host);
            drop(host);
            host = world::restore_program_checkpoint::<program::Program<CHILD_B>>(
                checkpoint, worker, false,
            );
        }
    }
}
#[test]
fn unchanged_parent_and_children_need_no_writer_in_the_edit() {
    exercise::<false>(false, false)
}
#[test]
fn worker_only_unchanged_parent_and_children_need_no_writer_in_the_edit() {
    exercise::<false>(true, false)
}
#[test]
fn a_healthy_parent_discovers_a_stale_child_in_the_same_edit() {
    exercise::<true>(false, false)
}
#[test]
fn worker_only_a_healthy_parent_discovers_a_stale_child_in_the_same_edit() {
    exercise::<true>(true, false)
}
#[test]
fn checkpoint_parent_and_children_need_no_writer_in_the_edit() {
    exercise::<false>(false, true)
}
#[test]
fn worker_only_checkpoint_parent_and_children_need_no_writer_in_the_edit() {
    exercise::<false>(true, true)
}
#[test]
fn a_checkpoint_parent_discovers_its_stale_child_in_the_edit() {
    exercise::<true>(false, true)
}
#[test]
fn worker_only_a_checkpoint_parent_discovers_its_stale_child_in_the_edit() {
    exercise::<true>(true, true)
}

mod standalone;
