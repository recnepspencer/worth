//! One fixed edit retains and discharges both actual failed native writer owners.
use super::super::super::super::program::Entry;
use super::*;
type Action = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    edit::Binding,
    unstable::program::Delivery,
>;
fn exercise(worker: bool, interrupted: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<unstable::program::Program>(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original_head = courts::native_head(&host);
    let before = super::super::super::failure::checkpoint(&host);
    let key = "owned-native-cleanups".to_owned();
    let operation = request
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, cause)| panic!("key capture: {cause:?}"))
        .retain_operation::<unstable::program::Program, Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("retention: {cause:?}"));
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let schedule = host
        .runtime()
        .schedule_managed_cleanups_on_worker_for_test(std::num::NonZeroUsize::new(2).unwrap())
        .unwrap();
    let WorthQueryOperationAdvance::Terminal {
        operation: mut terminal,
        denial,
    } = operation.advance(&request)
    else {
        panic!("the complete edit must refuse with owned cleanup")
    };
    assert!(
        matches!(denial, WorthQueryOperationAdvanceDenial::NativeTerminal),
        "{denial:?}"
    );
    let Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(cause),
    )) = terminal.native_outcome()
    else {
        panic!("original whole-edit refusal")
    };
    assert_eq!(cause.invariant_execution_failure().unwrap().kind(),worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert!(cause
        .invariant_execution_failure()
        .unwrap()
        .detail()
        .contains("owned-outputs.unstable-discovery-binding.v1"));
    let failures = terminal.managed_cleanup_failures();
    assert_eq!(
        failures.len(),
        2,
        "both actual writers failed native cleanup"
    );
    let identities=failures.map(|failure| {
        worth_query_testkit_certification_operation::facade::certification::source_only::managed_cleanup::assert_native_failure(failure);
        failure.run_identity().to_owned()
    }).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(identities.len(), 2, "the retained owners are distinct");
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2
    );
    assert_eq!(courts::native_head(&host), original_head);
    super::super::super::failure::assert_model_unchanged(&host, &before);
    drop(schedule);
    if interrupted {
        use worth_query_host::facade::admission::authenticated_principal::{
            WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
        };
        let cancellation = WorthQueryCancellationSource::new();
        let interrupted_scope = WorthQueryRequestScope::new(
            None,
            cancellation.token(),
            WorthQueryAdmissionClock::system(),
        );
        let interrupted_principal =
            authenticate_operator(host.installed_schema(), &interrupted_scope);
        let interrupted_request = host.request(&interrupted_principal, &interrupted_scope);
        let control = terminal
            .cancel_after_managed_releases_for_test(std::num::NonZeroUsize::MIN, cancellation)
            .unwrap();
        assert!(
            terminal
                .cancel_after_managed_releases_for_test(
                    std::num::NonZeroUsize::MIN,
                    WorthQueryCancellationSource::new()
                )
                .is_err(),
            "the active receiver registration cannot be replaced"
        );
        let (remaining, denial) = terminal
            .close_with_evidence(&interrupted_request)
            .err()
            .expect("cancellation after the first actual native release refuses close");
        assert!(
            matches!(
                denial,
                WorthQueryOperationCloseDenial::Execution(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                )
            ),
            "{denial:?}"
        );
        assert_eq!(
            remaining.managed_cleanup_failures().len(),
            1,
            "the first owner really released, the other remains owned"
        );
        assert_eq!(courts::native_head(&host), original_head);
        super::super::super::failure::assert_model_unchanged(&host, &before);
        drop(control);
        let mut remaining = remaining;
        let _replacement = remaining
            .cancel_after_managed_releases_for_test(
                std::num::NonZeroUsize::new(2).unwrap(),
                WorthQueryCancellationSource::new(),
            )
            .unwrap();
        terminal = remaining;
    }
    let closed = terminal
        .close_with_evidence(&request)
        .unwrap_or_else(|(_, cause)| {
            panic!("lawful close retains all remaining owners: {cause:?}")
        });
    assert!(
        closed.source_result().is_none(),
        "refused edits expose no unpublished handler result"
    );
    let receipts = closed.close_evidence().managed_cleanup_receipts();
    assert_eq!(receipts.len(), 2);
    let released = receipts
        .iter()
        .map(|receipt| {
            let native = receipt.inspection();
            assert!(native.resources_released());
            assert_eq!(native.provider_work().provider_retained_bytes(), 0);
            native.run_identity().to_owned()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        released, identities,
        "every receipt belongs to its original native owner"
    );
    let Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::Denied(cause),
    )) = closed.source_outcome()
    else {
        panic!("closed diagnostics retain the refusal")
    };
    assert_eq!(cause.invariant_execution_failure().unwrap().kind(),worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
    assert_eq!(
        host.runtime().world_active_publication_attempts_for_test(),
        0
    );
    assert_eq!(courts::native_head(&host), original_head);
    super::super::super::failure::assert_model_unchanged(&host, &before);
}
#[test]
fn one_fixed_edit_retains_both_native_cleanup_owners_and_receipts() {
    exercise(false, false)
}
#[test]
fn worker_only_one_fixed_edit_retains_both_native_cleanup_owners_and_receipts() {
    exercise(true, false)
}

#[test]
fn an_interrupted_fixed_close_retains_its_released_prefix_and_remaining_owner() {
    exercise(false, true)
}
#[test]
fn worker_only_interrupted_fixed_close_retains_its_released_prefix_and_remaining_owner() {
    exercise(true, true)
}
