//! Independent outputs use actual Native succession and retain their identities.
use super::*;
mod program;
fn exercise(worker: bool, cold: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let mut host = world::install_program_with_value::<program::Program>(limits, 2);
    let scope = request_scope();
    let mut original = None;
    for (ordinal, value, expected) in [(0, 7, 1), (1, 9, 1), (2, 9, 0)] {
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let before = courts::native_head(&host).0;
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let outcome = request
            .advanced()
            .mutate(member_edit::Intent {
                key: "m".into(),
                value,
            })
            .without_source()
            .idempotency(&format!("independent-edit-{ordinal}"))
            .execute_performed::<program::Program, program::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let done = match outcome {
            WorthQueryApplicationPerformedMutationOutcome::Performed(done) => done,
            WorthQueryApplicationPerformedMutationOutcome::NotPerformed(cause) => {
                panic!("independent edit {ordinal} value {value} refused: {cause:?}")
            }
            WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(cause) => {
                panic!(
                    "independent edit {ordinal} value {value} lost its output: {:?}",
                    cause.denial()
                )
            }
            _ => panic!("independent edit {ordinal} value {value} did not perform"),
        };
        let row = request
            .query(query::child_output::Read("child:m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(row.value, value * 100);
        if let Some(id) = original {
            assert_eq!(row.native_id, id);
        } else {
            original = Some(row.native_id);
        }
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected
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
            contacts + expected
        );
        assert_eq!(courts::native_head(&host), head);
        if cold && ordinal == 0 {
            drop(outputs);
            let checkpoint = super::super::failure::checkpoint(&host);
            drop(host);
            host = world::restore_program_checkpoint::<program::Program>(checkpoint, worker, false);
        }
    }
}
#[test]
fn independent_create_preserve_and_healthy_reuse_are_single_edit_commits() {
    exercise(false, false)
}
#[test]
fn worker_only_independent_create_preserve_and_healthy_reuse_are_single_edit_commits() {
    exercise(true, false)
}
#[test]
fn cold_independent_routes_keep_their_original_output_identity() {
    exercise(false, true)
}
#[test]
fn worker_only_cold_independent_routes_keep_their_original_output_identity() {
    exercise(true, true)
}
