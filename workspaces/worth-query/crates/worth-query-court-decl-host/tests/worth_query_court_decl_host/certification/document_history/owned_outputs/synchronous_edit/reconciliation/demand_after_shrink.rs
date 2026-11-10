//! After a synchronous shrink performs, demanding the outputs refuses. Public #126.
//!
//! This court pins a defect, so it must fail when #126 is fixed. Traced at runtime:
//! delivery's hot adoption of the co-committed completion checks its receipt for
//! currentness, finds no sealed Native output witness on the record, and returns
//! `RetainedBasisUnavailable` where only `Superseded` falls through to a real run
//! (`coupled_adoption::select`, `currentness/facts.rs::require_witnessed_output`).
//!
//! What recovers the program, measured once: a restart from a checkpoint (the
//! demand then settles with 2 contacts and 1 commit) and a later edit that grows
//! the fan-out (the demand then settles with no contact). The same demand again,
//! the operation-style demand and a further shrink all refuse the same way.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

fn refuses(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Synchronous>(limits, 5);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let demand = || {
        request
            .advanced()
            .start_program_outputs::<program::Synchronous, program::Tree>(
                &host,
                Demand("a".into()),
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap()
    };
    let mut initial = demand();
    let mut settled = false;
    for _ in 0..128 {
        if matches!(
            initial.advance(&host, &request).unwrap(),
            WorthQueryApplicationProgramOutputProgress::Settled(_)
        ) {
            settled = true;
            break;
        }
    }
    assert!(settled, "the initial fan-out settles before the shrink");
    drop(initial);
    let (outcome, work) = observe_maintenance_work(64, || {
        request
            .advanced()
            .mutate(edit::Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"demand-after-synchronous-shrink".to_owned())
            .execute_performed::<program::Synchronous, program::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ),
        "the shrink itself performs"
    );
    assert_eq!(
        work.published_rows.len() + work.omitted_publications,
        1,
        "the shrink is one commit"
    );
    drop(outcome);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (refusal, work) = observe_maintenance_work(64, || {
        let mut after = demand();
        after.advance(&host, &request).err()
    });
    let refusal = refusal.expect(
        "public #126 is fixed if this demand no longer refuses: replace this court with the settled journey",
    );
    // The refusal's typed kind is nested in the handle's own error; its rendering is stable.
    let rendered = format!("{refusal:?}");
    assert!(
        rendered.contains("kind: RetainedBasisUnavailable")
            && rendered.contains("settled output is not current at the selected observation")
            && rendered.contains("recovery_posture: Terminal"),
        "{rendered}"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "the refused demand contacts no producer"
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
}
#[test]
fn a_demand_after_a_synchronous_shrink_refuses_until_public_126_is_fixed() {
    refuses(false);
}
#[test]
fn worker_only_demand_after_a_synchronous_shrink_refuses_until_public_126_is_fixed() {
    refuses(true);
}
