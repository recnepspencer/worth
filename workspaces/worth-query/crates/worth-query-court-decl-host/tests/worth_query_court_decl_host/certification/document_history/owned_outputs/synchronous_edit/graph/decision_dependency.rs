//! A retained source query is unchanged, while an actual producer decision input moves at D.
use super::*;
mod program;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Program>(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut original = None;
    for value in [5, 9] {
        let before = courts::native_head(&host).0;
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let outcome = request
            .advanced()
            .mutate(member_edit::Intent {
                key: "m".into(),
                value,
            })
            .without_source()
            .idempotency(&format!("decision-dependency-{value}"))
            .execute_performed::<program::Program, program::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = outcome else {
            panic!("decision dependency edit must perform")
        };
        let row = request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(
            row.value,
            2 + value,
            "the real producer must read its changed decision input at D"
        );
        if let Some(id) = original {
            assert_eq!(row.native_id, id);
        } else {
            original = Some(row.native_id);
        }
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 1
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
            contacts + 1
        );
        assert_eq!(courts::native_head(&host), head);
    }
}
#[test]
fn a_decision_input_changed_only_in_the_candidate_runs_the_real_producer() {
    exercise(false)
}
#[test]
fn worker_only_a_decision_input_changed_only_in_the_candidate_runs_the_real_producer() {
    exercise(true)
}
