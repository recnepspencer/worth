//! A genuine ordinary Ready is reused without manufacturing a candidate epoch.
use super::*;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host =
        world::install_program_with_other_model::<program::Program<false>>(limits, 3, Some(5));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut operation = request
        .demand(first::Demand("a".into()))
        .retain_program_outputs::<program::Program<false>, program::Tree<false>>(&host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    let mut settled = false;
    for _ in 0..128 {
        operation = match operation.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(&request).unwrap();
                done.close(&request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                settled = true;
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        };
    }
    assert!(settled);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    let first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let second = request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!((first.value, second.value), (30, 1003));
    let before = courts::native_head(&host).0;
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let result = request
        .advanced()
        .mutate(edit::Intent {
            key: "b".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"ordinary-ready-unrelated-edit".to_owned())
        .execute_performed::<program::Program<false>, program::Tree<false>>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = result else {
        panic!("unrelated source edit must perform")
    };
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "original ordinary Ready reuses both healthy producers"
    );
    assert_eq!(courts::native_head(&host).1, [before]);
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0],
        first
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0],
        second
    );
    let head = courts::native_head(&host);
    let (_, _, mut outputs) = done
        .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
        .into_parts();
    let mut settled = false;
    for _ in 0..128 {
        if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
            outputs.advance(&host, &request).unwrap()
        {
            settled = true;
            break;
        }
    }
    assert!(settled);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(courts::native_head(&host), head);
}
#[test]
fn ordinary_ready_keeps_a_healthy_parent_and_child_unopened() {
    exercise(false)
}
#[test]
fn worker_only_ordinary_ready_keeps_a_healthy_parent_and_child_unopened() {
    exercise(true)
}

fn published_without_ready(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host =
        world::install_program_with_other_model::<program::Program<false>>(limits, 3, Some(5));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut original = request
        .demand(first::Demand("a".into()))
        .start_in_program::<program::Program<false>, program::Tree<false>>(&host)
        .unwrap();
    // Pause the existing owner after Native publication, before it issues Ready.
    host.runtime()
        .delay_next_output_readiness_delivery_for_test();
    let mut published = false;
    for _ in 0..128 {
        let progress = original.advance(&request).unwrap();
        let exists = request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .is_ok_and(|output| !output.rows().is_empty());
        if exists {
            assert!(
                !matches!(
                    progress,
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                ),
                "stop at actual native publication, before Ready settlement"
            );
            published = true;
            break;
        }
    }
    assert!(published);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    assert_eq!(contacts, 1);
    let row = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(row.value, 30);
    let before = courts::native_head(&host).0;
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "b".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"no-ready-yet".to_owned())
        .execute_performed::<program::Program<false>, program::Tree<false>>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = outcome else {
        panic!("the real fallback and child must co-commit")
    };
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "a publication without committed Ready cannot stand in for its result"
    );
    assert_eq!(courts::native_head(&host).1, [before]);
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0],
        row
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1003
    );
    let head = courts::native_head(&host);
    let (_, _, mut outputs) = done
        .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
        .into_parts();
    let mut settled = false;
    for _ in 0..128 {
        if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
            outputs.advance(&host, &request).unwrap()
        {
            settled = true;
            break;
        }
    }
    assert!(settled);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    assert_eq!(courts::native_head(&host), head);
    drop(original);
}
#[test]
fn a_published_output_without_ready_takes_the_real_producer_fallback() {
    published_without_ready(false)
}
#[test]
fn worker_only_a_published_output_without_ready_takes_the_real_producer_fallback() {
    published_without_ready(true)
}
