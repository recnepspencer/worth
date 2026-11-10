//! One edit starts one producer per discovered source; the attempt ceiling counts each start.
use super::*;
mod ownership_reads;
pub(super) mod program;
mod touch_work;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::ApplicationProducerRoundLimits;
use worth_relational::facade::inspection::observe_maintenance_work;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
/// The fixture world seeds its decision member with this value.
const MEMBER_VALUE: u64 = 5;
const ROUTES: u64 = 3;
fn source_key(ordinal: u64) -> String {
    format!("k{ordinal}")
}
pub(super) fn install(worker: bool) -> Host {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    world::install_program_with_value::<program::Program>(limits, 2)
}
/// Each seeded source is a real authored model with its own committed producers.
pub(super) fn seed(host: &Host, request: &family::Request<'_, '_, '_>, count: u64) {
    for ordinal in 0..count {
        let seeded = request
            .advanced()
            .mutate(new_source::Intent {
                key: source_key(ordinal),
                value: ordinal,
            })
            .without_source()
            .idempotency(&format!("many-routes-seed-{ordinal}"))
            .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                seeded,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{seeded:?}"
        );
    }
}
pub(super) fn fan_out(
    host: &Host,
    request: &family::Request<'_, '_, '_>,
    routes: u64,
    total: usize,
) -> WorthQueryApplicationPerformedMutationOutcome<
    Schema,
    edit::Binding,
    program::Program,
    program::Tree,
> {
    request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: routes,
        })
        .producer_round_limits(ApplicationProducerRoundLimits::new(
            NonZeroUsize::new(64).unwrap(),
            NonZeroUsize::new(total).unwrap(),
        ))
        .without_source()
        .idempotency(&format!("many-routes-{routes}-{total}"))
        .execute_performed::<program::Program, program::Tree>(
            host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}
fn parity(request: &family::Request<'_, '_, '_>, ordinal: u64) -> Option<u64> {
    request
        .query(query::first_output::Read(format!(
            "parity:{}",
            source_key(ordinal)
        )))
        .execute()
        .ok()
        .and_then(|read| read.rows().first().map(|row| row.value))
}

fn the_last_allowed_start_completes(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    seed(&host, &request, ROUTES);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, work) =
        observe_maintenance_work(8, || fan_out(&host, &request, ROUTES, ROUTES as usize + 1));
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ),
        "the parent and every discovered source fit the exact total"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + ROUTES + 1
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 1);
    for ordinal in 0..ROUTES {
        assert_eq!(
            parity(&request, ordinal),
            Some(ordinal % 2 + MEMBER_VALUE),
            "every discovered source published its output in the one commit"
        );
    }
}
#[test]
fn the_exact_attempt_total_admits_every_discovered_route() {
    the_last_allowed_start_completes(false)
}
#[test]
fn worker_only_exact_attempt_total_admits_every_discovered_route() {
    the_last_allowed_start_completes(true)
}

fn one_start_too_many_refuses(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    seed(&host, &request, ROUTES);
    let seeded = (0..ROUTES)
        .map(|ordinal| {
            request
                .query(query::model::Read(source_key(ordinal)))
                .execute()
                .unwrap()
                .rows()[0]
                .native_id
        })
        .collect::<Vec<_>>();
    let before = super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, work) =
        observe_maintenance_work(8, || fan_out(&host, &request, ROUTES, ROUTES as usize));
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            refusal,
        )),
    ) = outcome
    else {
        panic!("the start past the attempt total must refuse the whole edit")
    };
    let cause = refusal
        .coupled_producer_denial()
        .expect("the original producer budget refusal");
    let WorthQueryOutputDemandDenialKind::ProducerAttemptRoundBudgetExceeded {
        source,
        started,
        limit,
    } = cause.kind()
    else {
        panic!("expected the attempt round budget, got {cause:?}")
    };
    assert_eq!((started, limit), (ROUTES as usize, ROUTES as usize));
    assert!(
        seeded.contains(&source),
        "the refusal names the discovered source whose start was refused"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + ROUTES,
        "the refused start never contacts its handler"
    );
    assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
    assert_eq!(courts::native_head(&host), head);
    super::super::failure::assert_model_unchanged(&host, &before);
    for ordinal in 0..ROUTES {
        assert_eq!(parity(&request, ordinal), None);
    }
}
#[test]
fn one_start_past_the_attempt_total_refuses_every_discovered_route() {
    one_start_too_many_refuses(false)
}
#[test]
fn worker_only_one_start_past_the_attempt_total_refuses_every_discovered_route() {
    one_start_too_many_refuses(true)
}
