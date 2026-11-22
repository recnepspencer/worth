//! Discovery support remains live through the final candidate, including own producers.
use super::*;
#[path = "support_program.rs"]
mod support_program;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host =
        world::install_program_with_other_model::<support_program::Program>(limits, 2, Some(3));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::failure::checkpoint(&host);
    let source = request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"invalidated-root-query".to_owned())
        .execute_performed_discovered::<support_program::Program, support_program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("the discovered set must remain justified by the final candidate")
    };
    let refusal = cause
        .invariant_execution_failure()
        .expect("the final support owner refuses");
    assert_eq!(
        refusal.kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
    );
    assert!(
        refusal
            .detail()
            .contains("owned-outputs.member-public-binding.v1")
            && refusal.detail().contains("MemberValue"),
        "{refusal:?}"
    );
    assert_eq!(
        refusal.affected_records(),
        [worth_relational::facade::transactions::RecordRef::Entity(
            source
        )]
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    super::super::super::failure::assert_model_unchanged(&host, &before);
    assert!(request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .is_err());
}
#[test]
fn a_discovered_producer_cannot_invalidate_its_own_root_discovery() {
    exercise(false)
}
#[test]
fn worker_only_a_discovered_producer_cannot_invalidate_its_own_root_discovery() {
    exercise(true)
}
/// The same root as the envelope's only (prefix) stage (#224): its producer is
/// owed at the prefix's own position and changes the member that selected
/// route `a`. No position settles a selection, so the envelope refuses
/// atomically, attributed to that producer, with no #229 hold.
#[test]
fn an_ordered_prefix_producer_cannot_invalidate_its_own_root_selection() {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<support_program::Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(3),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::failure::checkpoint(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("ordered-invalidated-root-query".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<support_program::Tree, _>(edit::Intent {
        key: "a".into(),
        value: 7,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("{outcome:?}")
    };
    let refusal = cause
        .invariant_execution_failure()
        .expect("the final support owner refuses");
    assert_eq!(
        refusal.kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
    );
    assert!(
        refusal
            .detail()
            .contains("owned-outputs.member-public-binding.v1")
            && refusal.detail().contains("later writer producer"),
        "{refusal:?}"
    );
    assert_eq!(refusal.coupled_support_hold(), None);
    super::super::super::failure::assert_model_unchanged(&host, &before);
    assert!(request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .is_err());
}

/// A root selection's stage window (#238) ends at the next authored stage by
/// owed order, not physically. Stage 0's root selects decision-write(a),
/// which rewrites member `m`, the root's own support. Base order runs that
/// producer in the final drive, physically after stage 1 (b = 9, the same
/// root). It is still owed by stage 0, so it lies inside root 0's window: one
/// commit per stage refuses commit 0, and so does the envelope, attributed to
/// decision-write, with no hold. (Root 1, which reads `m` too, would refuse
/// as well; no public fixture's later stage roots a different tree, so this
/// court pins the outcome rather than isolating root 0's window.)
#[test]
fn a_root_selections_own_producer_owed_by_its_stage_stays_inside_its_window() {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<support_program::Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(3),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::failure::checkpoint(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("windowed-invalidated-root-query".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<support_program::Tree, _>(edit::Intent {
        key: "a".into(),
        value: 7,
    })
    .unwrap();
    edit.push_stage::<support_program::Tree, _>(edit::Intent {
        key: "b".into(),
        value: 9,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("{outcome:?}")
    };
    let refusal = cause
        .invariant_execution_failure()
        .expect("the final support owner refuses");
    assert_eq!(
        refusal.kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
    );
    assert!(
        refusal.detail().contains("later writer producer owned-outputs.decision-write"),
        "{refusal:?}"
    );
    assert_eq!(refusal.coupled_support_hold(), None);
    super::super::super::failure::assert_model_unchanged(&host, &before);
}
