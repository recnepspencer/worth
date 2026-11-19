//! A later contribution cannot invalidate a verified parent and still publish the edit.
use super::*;
mod derived;
mod program;
#[macro_use]
mod staged;
mod prerequisite;
/// Commits `b = 4` and settles its outputs: `first:a` is then a verified
/// parent derived from model `a` and member `m`, and `second:b` its child.
fn settle_initial(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Program>,
    request: &family::Request<'_, '_, '_>,
) {
    let initial_contacts = host.producer_contacts_on_this_thread_for_test();
    let first = request
        .advanced()
        .mutate(edit::Intent {
            key: "b".into(),
            value: 4,
        })
        .without_source()
        .idempotency(&"late-parent-initial".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = first else {
        panic!("same-value child write must preserve the initial parent support")
    };
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        initial_contacts + 2
    );
    let (_, _, mut outputs) = done
        .start_required_outputs(host, request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
        .into_parts();
    let mut settled = false;
    for _ in 0..128 {
        if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
            outputs.advance(host, request).unwrap()
        {
            settled = true;
            break;
        }
    }
    assert!(settled);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        initial_contacts + 2
    );
    drop(outputs);
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        7
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:b".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1004
    );
}
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_other_model::<program::Program>(limits, 2, Some(7));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_initial(&host, &request);
    let before = super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "b".into(),
            value: 8,
        })
        .without_source()
        .idempotency(&"late-parent-refused".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("changed verified-parent support must refuse the whole edit")
    };
    let failure = cause
        .invariant_execution_failure()
        .expect("the final candidate read-support owner refuses");
    assert_eq!(failure.kind(), worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert!(
        failure.detail().contains("decision-input")
            && failure.detail().contains("verified output support")
            && failure.detail().contains("MemberValue"),
        "{failure:?}"
    );
    // No authored stage read the parent, so nothing names an open issue (#244).
    assert_eq!(failure.coupled_support_hold(), None, "{failure:?}");
    let model = |key: &str| {
        worth_relational::facade::transactions::RecordRef::Entity(
            request.query(query::model::Read(key.into())).execute().unwrap().rows()[0].native_id,
        )
    };
    // The reader's source, then the exact writer: the child producer of `b`.
    assert_eq!(failure.affected_records(), [model("a"), model("b")]);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "the healthy original parent stays unopened"
    );
    assert_eq!(courts::native_head(&host), head);
    super::super::failure::assert_model_unchanged(&host, &before);
}
#[test]
fn a_later_child_cannot_invalidate_the_verified_parent_before_publication() {
    exercise(false)
}
#[test]
fn worker_only_a_later_child_cannot_invalidate_the_verified_parent_before_publication() {
    exercise(true)
}
/// The same change as one authored stage of an ordered edit (#244): the
/// stage's child `decision-write(b)` changes `m`, which the verified parent
/// `first:a` read. That child is selected by a discovery that reads `first:a`,
/// but it has no prerequisite supplier and derives from model `b` alone.
/// Selection is not consumption, so the edit refuses whole with a plain
/// CoupledReadSupportChanged, attributed to the child, and nothing commits.
/// The consuming variant is `prerequisite`.
#[test]
fn an_authored_stage_whose_discovered_child_changes_its_verified_parent_is_not_held() {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<program::Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_initial(&host, &request);
    let before = super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-own-producer".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("{outcome:?}")
    };
    let failure = cause
        .invariant_execution_failure()
        .expect("the final support owner refuses");
    assert_eq!(failure.kind(), worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert_eq!(failure.coupled_support_hold(), None, "{failure:?}");
    assert!(
        failure.detail().contains("authored stage 0")
            && failure.detail().contains("later writer producer owned-outputs.decision-write")
            && failure.detail().contains("verified output support")
            && failure.detail().contains("MemberValue"),
        "{failure:?}"
    );
    let model = |key: &str| {
        worth_relational::facade::transactions::RecordRef::Entity(
            request.query(query::model::Read(key.into())).execute().unwrap().rows()[0].native_id,
        )
    };
    assert_eq!(failure.affected_records(), [model("a"), model("b")]);
    assert_eq!(courts::native_head(&host), head);
    super::super::failure::assert_model_unchanged(&host, &before);
}
