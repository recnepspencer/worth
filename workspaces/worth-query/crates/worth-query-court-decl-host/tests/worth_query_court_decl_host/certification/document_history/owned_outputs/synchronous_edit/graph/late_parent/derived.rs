//! The parent `first:a` is derived inside the edit, not verified, and a later
//! producer changes the member `m` its derivation read (#246). Under D3 the
//! edit re-derives `first:a` from the new `m` and commits; the sequential
//! world, which commits the write alone and derives `first:a` on demand,
//! answers the same value.
use super::staged::{NonConsumingProgram, ParentTree, WriteTree};
use super::*;

/// What the D3 courts compare: `first:a`, derived as model `a` (2) plus the
/// member `m` that `decision-write(b)` writes as model `b` + 1.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct World {
    pub(super) first: u64,
}

pub(super) fn observe(request: &family::Request<'_, '_, '_>) -> World {
    World {
        first: request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
    }
}

/// The edit commits whole.
pub(super) fn committed(
    outcome: worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome,
) {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
}

/// The sequential oracle: `decision-write(b)` commits alone (`b = 8`), then
/// `first:a` is derived on demand from the committed `m`.
pub(super) fn sequential() -> World {
    let host = world::install_program_with_other_model::<NonConsumingProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-derived-sequential".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<WriteTree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    committed(
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    );
    let mut operation = request
        .demand(family::decision_input::Demand("a".into()))
        .retain_program_outputs::<NonConsumingProgram, ParentTree>(&host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    for _ in 0..128 {
        operation = match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(&request).unwrap();
                done.close(&request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                return observe(&request);
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("{denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        };
    }
    panic!("on-demand first:a did not settle")
}

/// One authored stage (`b = 8`) enters `decision-input(a)`, which derives
/// `first:a` inside the edit from model `a` and member `m` (2 + 5). Its child
/// `decision-write(b)`, selected by a discovery that reads `first:a` but with
/// no prerequisite supplier, derives from model `b` alone and changes `m`.
/// Qualified use reopens `first:a`, its successor round re-derives it from
/// `m = 9` (2 + 9), and the edit commits the sequential world (D3).
#[test]
fn a_derived_parents_discovered_child_changing_its_decision_read_rederives_it() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<program::Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-derived-consumer".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    committed(
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    );
    let world = observe(&request);
    assert_eq!(world.first, 11, "first:a re-derived from m = 9");
    assert_eq!(world, sequential());
}

/// Stage 0 (`a = 2`) enters `decision-input(a)`, which derives `first:a`
/// inside the edit from model `a` and member `m`; stage 1 (`b = 8`) enters
/// `decision-write(b)` straight from the edit, which changes `m`. That writer
/// never consumed `first:a`; qualified use reopens `first:a` alone, and the
/// edit commits the sequential world (D3).
#[test]
fn an_unrelated_producer_changing_a_derived_parents_decision_read_rederives_it() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<NonConsumingProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-derived-unrelated".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<ParentTree, _>(edit::Intent {
        key: "a".into(),
        value: 2,
    })
    .unwrap();
    edit.push_stage::<WriteTree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    committed(
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    );
    let world = observe(&request);
    assert_eq!(world.first, 11, "first:a re-derived from m = 9");
    assert_eq!(world, sequential());
}
