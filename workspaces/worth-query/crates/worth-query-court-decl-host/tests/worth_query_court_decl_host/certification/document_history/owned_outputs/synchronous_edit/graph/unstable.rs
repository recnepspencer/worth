//! Discovery support changed by a later child must refuse before publication:
//! the parent selects again and drops the child it derived (#238, PR C C4).
use super::*;
pub(super) mod program;
pub mod query;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> =
        world::install_program_with_value(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"unstable-discovery".to_owned())
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
        panic!("changed discovery support must refuse the whole edit")
    };
    // The child's member empties the parent's discovery, which selected it:
    // the parent selects again (#238), drops the child it already derived,
    // and the wave refuses that before any publication.
    let source = request
        .query(super::super::super::query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let denial = cause
        .coupled_producer_denial()
        .unwrap_or_else(|| panic!("the wave refuses the dropped selection: {cause:?}"));
    assert!(
        matches!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::SupersededSelectionDropped { source: dropped }
                if dropped == source
        ),
        "{denial:?}"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    super::super::failure::assert_model_unchanged(&host, &before);
}
#[test]
fn a_child_that_changes_discovery_support_refuses_the_whole_edit() {
    exercise(false)
}
#[test]
fn worker_only_child_that_changes_discovery_support_refuses_the_whole_edit() {
    exercise(true)
}
