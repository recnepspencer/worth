//! Deleting and navigating a generic authored member exercises actual historical scope admission.
use super::*;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
fn published(worker: bool) -> Host {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: Host = world::install_program_with_value(limits, 1);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut outputs = request
        .demand(downstream::Demand("m".into()))
        .retain_program_outputs::<program::Program, program::Tree>(&host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    loop {
        outputs = match outputs.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(&request).unwrap();
                done.close(&request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => {
                panic!("initial tree terminated")
            }
        };
    }
    host
}
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = published(worker);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = request
        .query(query::child_output::Read("downstream:m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let child = request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let document = world::attach(&host);
    let before_delete = document.current_step(&request).unwrap().id();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    downstream::Demand("m".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let mut deletion = document
        .edit(authored_member::retire::Intent)
        .idempotency_owned(&request, "historical-parent-delete".into())
        .unwrap()
        .retain_in_program::<program::Program, super::super::super::program::AuthoredRetireAction>(
            &host,
        )
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    let mut advances = 0;
    loop {
        advances += 1;
        assert!(
            advances < 512,
            "retirement did not progress: {:?}",
            deletion.posture()
        );
        deletion = match deletion.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                deletion = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = deletion
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    if let WorthQueryHistoryEditOutcome::NotRecorded { terminal, .. } = &outcome {
        panic!(
            "delete was not recorded: {:?}; admission: {:?}",
            terminal.native_outcome(),
            terminal.native_admission_denial()
        );
    }
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let current = document.selection(&request).unwrap();
    let at_c = host
        .request(&principal, &scope)
        .on_branch(current.physical_endpoint().product_branch());
    assert!(at_c
        .query(query::member_public::Read("m".into()))
        .execute()
        .is_err());
    assert!(at_c
        .query(query::child_output::Read("downstream:m".into()))
        .execute()
        .is_err());
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let selected = super::super::super::courts::checkout(
        &document,
        &request,
        before_delete,
        current,
        0x0d12_1420,
    )
    .unwrap_or_else(|cause| panic!("historical-only non-leaf discovery: {cause:?}"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "healthy roots and children are selected without producer contacts"
    );
    let at_d = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch());
    assert_eq!(
        at_d.query(query::child_output::Read("downstream:m".into()))
            .execute()
            .unwrap()
            .rows(),
        &[original]
    );
    assert_eq!(
        at_d.query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows(),
        &[child]
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn restored_non_leaf_discovers_its_child_when_its_output_scope_is_absent_at_c() {
    exercise(false);
}
#[test]
fn worker_only_restored_non_leaf_discovers_its_child_when_its_output_scope_is_absent_at_c() {
    exercise(true);
}

#[test]
fn an_installed_authored_retirement_retires_the_published_member() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = published(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(authored_member::retire::Intent)
        .without_source()
        .idempotency(&String::from("direct-authored-member-retire"))
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
}
