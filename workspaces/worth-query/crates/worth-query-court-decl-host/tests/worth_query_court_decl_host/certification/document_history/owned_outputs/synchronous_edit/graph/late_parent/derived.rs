//! The parent `first:a` is derived inside the edit, not verified, and a later
//! producer changes the member `m` its derivation read (#246).
use super::staged::{NonConsumingProgram, ParentTree, WriteTree};
use super::*;

/// The whole edit refuses on `first:a`'s decision support, attributed to the
/// `decision-write(b)` writer, with exactly `hold`, and nothing commits.
fn refuse_whole(
    request: &family::Request<'_, '_, '_>,
    outcome: worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome,
    hold: Option<worth_query_host::facade::primary_graph::WorthQueryCoupledSupportHold>,
) {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("{outcome:?}")
    };
    let failure = cause
        .invariant_execution_failure()
        .unwrap_or_else(|| panic!("the final support owner refuses: {cause:?}"));
    assert_eq!(failure.kind(), worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert_eq!(failure.coupled_support_hold(), hold, "{failure:?}");
    assert!(
        failure
            .detail()
            .contains("later writer producer owned-outputs.decision-write")
            && failure.detail().contains("changed decision support"),
        "{failure:?}"
    );
    let model = |key: &str| {
        worth_relational::facade::transactions::RecordRef::Entity(
            request
                .query(query::model::Read(key.into()))
                .execute()
                .unwrap()
                .rows()[0]
                .native_id,
        )
    };
    // The reader's source, then the exact writer's source.
    assert_eq!(failure.affected_records(), [model("a"), model("b")]);
}

/// One authored stage (`b = 8`) enters `decision-input(a)`, which derives
/// `first:a` inside the edit from model `a` and member `m`. Its child
/// `decision-write(b)`, selected by a discovery that reads `first:a` but with
/// no prerequisite supplier, derives from model `b` alone and changes `m`.
/// Selection is not consumption, so the edit refuses whole with a plain
/// CoupledReadSupportChanged. The consuming variant is `prerequisite`.
#[test]
fn a_derived_parents_discovered_child_changing_its_decision_read_is_not_held() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<program::Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
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
    refuse_whole(
        &request,
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        None,
    );
    assert_eq!(courts::native_head(&host), head);
}

/// Stage 0 (`a = 2`) enters `decision-input(a)`, which derives `first:a`
/// inside the edit from model `a` and member `m`; stage 1 (`b = 8`) enters
/// `decision-write(b)` straight from the edit, which changes `m`. That writer
/// never consumed `first:a`, so the edit refuses whole with a plain
/// CoupledReadSupportChanged.
#[test]
fn an_unrelated_producer_changing_a_derived_parents_decision_read_is_not_held() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<NonConsumingProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
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
    refuse_whole(
        &request,
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        None,
    );
    assert_eq!(courts::native_head(&host), head);
}
