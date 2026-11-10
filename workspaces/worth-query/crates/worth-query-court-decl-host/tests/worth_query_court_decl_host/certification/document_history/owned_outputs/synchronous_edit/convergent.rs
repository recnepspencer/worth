//! A real owned producer converges through two effectful Preserve executions.
use super::*;
use family::convergent as producer;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;
mod program;
fn settle<P>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
    request: &family::Request<'_, '_, '_>,
) where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let mut operation = request
        .demand(producer::Demand("a".into()))
        .retain_program_outputs::<P, program::Tree>(host)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
    for _ in 0..128 {
        operation = match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(request).unwrap();
                done.close(request)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(t) => panic!("{:?}", t.denial()),
        }
    }
    panic!("real convergent producer did not settle");
}
fn limits(worker: bool) -> WorthQueryInMemoryApplicationLimits {
    if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    }
}
fn row(request: &family::Request<'_, '_, '_>) -> query::Row {
    request
        .query(query::first_output::Read("convergent:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
fn deferred(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program::<program::Deferred>(limits(worker));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request);
    let original = row(&request);
    assert_eq!(original.value, 10);
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"convergent-source".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, work) = observe_maintenance_work(16, || settle(&host, &request));
    let final_row = row(&request);
    eprintln!(
        "CONVERGENT deferred-reference contacts={} commits={} final={}",
        host.producer_contacts_on_this_thread_for_test() - contacts,
        work.published_rows.len() + work.omitted_publications,
        final_row.value
    );
    assert_eq!(final_row.native_id, original.native_id);
    assert_eq!(
        final_row.value, 21,
        "the real successor consumes the initialized optional aspect"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        2
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 2);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, work) = observe_maintenance_work(16, || settle(&host, &request));
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
    assert_eq!(row(&request), final_row);
}
#[test]
fn a_convergent_owned_two_effect_route_keeps_the_real_successor() {
    deferred(false);
}
#[test]
fn worker_only_a_convergent_owned_two_effect_route_keeps_the_real_successor() {
    deferred(true);
}

fn synchronous(worker: bool, withheld: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program::<program::Synchronous>(limits(worker));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request);
    let original = row(&request);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    // Public #126: the route's record keeps facts without a sealed output witness.
    let withheld = withheld.then(|| host.withhold_next_coupled_output_witness_for_test());
    let (outcome, work) = observe_maintenance_work(16, || {
        request
            .advanced()
            .mutate(edit::Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"convergent-synchronous-source".to_owned())
            .execute_performed::<program::Synchronous, program::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = outcome else {
        panic!("the first effectful run must co-commit without refusing its real successor");
    };
    if let Some(withheld) = &withheld {
        assert!(
            withheld.taken_for_test(),
            "the route's witness was withheld"
        );
    }
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        1
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 1);
    let applied = row(&request);
    assert_eq!(applied.native_id, original.native_id);
    assert_eq!(
        applied.value, 20,
        "the first real effect is already applied with the edit"
    );
    let (_, _, mut outputs) = done
        .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|e| panic!("{:?}", e.denial()))
        .into_parts();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, work) = observe_maintenance_work(16, || {
        for _ in 0..128 {
            if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
                outputs.advance(&host, &request).unwrap()
            {
                return;
            }
        }
        panic!("the real effectful successor must settle");
    });
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        1
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 1);
    let final_row = row(&request);
    assert_eq!(final_row.native_id, original.native_id);
    assert_eq!(final_row.value, 21);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, work) = observe_maintenance_work(16, || settle(&host, &request));
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
    assert_eq!(row(&request), final_row);
}
#[test]
fn a_synchronous_owned_two_effect_route_keeps_its_delivery_successor() {
    synchronous(false, false);
}
#[test]
fn worker_only_synchronous_owned_two_effect_route_keeps_its_delivery_successor() {
    synchronous(true, false);
}
/// A born-stale record is never read for currentness, so it keeps its own-effect
/// successor whether or not its output witness was sealed.
#[test]
fn a_born_stale_route_without_its_output_witness_keeps_its_delivery_successor() {
    synchronous(false, true);
}
#[test]
fn worker_only_born_stale_route_without_its_output_witness_keeps_its_delivery_successor() {
    synchronous(true, true);
}
