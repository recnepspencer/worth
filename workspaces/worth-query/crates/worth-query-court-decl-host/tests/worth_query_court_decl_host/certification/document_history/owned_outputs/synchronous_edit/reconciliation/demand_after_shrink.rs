//! After a synchronous shrink performs, its outputs settle. Public #126.
//!
//! The shrink's co-committed record keeps comparable facts but has no sealed
//! Native output witness, so delivery's hot adoption could not check it for
//! currentness and refused `RetainedBasisUnavailable`, terminally. The record
//! now answers that it is not adoptable as current, and delivery runs the
//! producer on the ordinary path, as a restarted host does.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

type Host = WorthQueryProgramApplicationRuntime<Schema, program::Synchronous>;
type Outputs =
    WorthQueryApplicationProgramOutputHandle<Schema, program::Synchronous, program::Tree>;

fn demand(host: &Host, request: &family::Request<'_, '_, '_>) -> Outputs {
    request
        .advanced()
        .start_program_outputs::<program::Synchronous, program::Tree>(
            host,
            Demand("a".into()),
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap()
}
/// Settle the outputs and return the producer contacts and commits it took.
fn settle(
    host: &Host,
    request: &family::Request<'_, '_, '_>,
    mut outputs: Outputs,
) -> (u64, usize) {
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let ((), work) = observe_maintenance_work(64, || {
        for _ in 0..128 {
            if let WorthQueryApplicationProgramOutputProgress::Settled(_) = outputs
                .advance(host, request)
                .unwrap_or_else(|cause| panic!("{cause:?}"))
            {
                return;
            }
        }
        panic!("the outputs settle");
    });
    (
        host.producer_contacts_on_this_thread_for_test() - contacts,
        work.published_rows.len() + work.omitted_publications,
    )
}

fn journey(worker: bool, through_the_performed_handle: bool) {
    courts::with_legacy_rows(worker, |_, _, legacy| {
        let limits = if worker {
            world::worker_limits()
        } else {
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
        };
        let host = world::install_program_with_value::<program::Synchronous>(limits, 5);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        settle(&host, &request, demand(&host, &request));
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
        let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = outcome else {
            panic!("the shrink itself performs")
        };
        assert_eq!(
            work.published_rows.len() + work.omitted_publications,
            1,
            "the shrink is one commit"
        );
        let outputs = if through_the_performed_handle {
            done.start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
                .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
                .into_parts()
                .2
        } else {
            drop(done);
            demand(&host, &request)
        };
        assert_eq!(
            settle(&host, &request, outputs),
            (2, 1),
            "delivery runs the producer on the ordinary path, as a restarted host does"
        );
        let after = request
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(
            comparison::named(&after),
            comparison::named(&legacy),
            "the settled rows equal the sequential journey's"
        );
        assert_eq!(
            settle(&host, &request, demand(&host, &request)),
            (0, 0),
            "the settled outputs are then current"
        );
    });
}
#[test]
fn a_synchronous_shrinks_outputs_settle_through_its_performed_handle() {
    journey(false, true);
}
#[test]
fn a_synchronous_shrinks_outputs_settle_on_a_fresh_demand() {
    journey(false, false);
}
#[test]
fn worker_only_synchronous_shrinks_outputs_settle_through_its_performed_handle() {
    journey(true, true);
}
#[test]
fn worker_only_synchronous_shrinks_outputs_settle_on_a_fresh_demand() {
    journey(true, false);
}
