//! The actual policy receiver separates untouched state from a committed prefix.
use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{
        WorthQueryApplicationHistoryCloseFailure, WorthQueryDocumentHistoryClosingDenial,
        WorthQueryHistoryPageDenial,
    },
    product::{
        WorthQueryApplicationProductBranchCleanupDenial, WorthQueryProductBranchOwnerCleanupDenial,
    },
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{WorthQueryCancellationSource, WorthQueryRequestScope},
    presented_request::set_retention,
    readback::observe_head,
    settled_verdict::{settle, RetentionVerdict},
};

pub(super) fn filled(
    host: &App,
) -> (
    Document<'_>,
    worth_query_host::facade::application_entry::WorthQueryHistoryStep,
) {
    let mut document = attach(host, head(host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let root = document
        .current_step(&host.request(&principal, &scope))
        .unwrap();
    for (offset, days) in [3, 5, 6, 4].into_iter().enumerate() {
        document = finish(
            host,
            ready(
                host,
                begin(
                    host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_6600 + offset as u64,
                ),
            ),
            days,
        )
        .0;
    }
    (document, root)
}

fn cancelled<T>(result: Result<T, Failure>) {
    let Err(cause) = result else {
        panic!("cancelled delivery cannot succeed");
    };
    assert!(
        matches!(
            cause,
            Failure::Document(Native::Stopped(WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            )))
        ),
        "{cause:?}"
    );
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: cancellation_before_the_admitted_window_effect_preserves_eligibility_target_and_cursor joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn cancellation_before_the_admitted_window_effect_preserves_eligibility_target_and_cursor() {
    let host = publish_on_first_program();
    let (mut document, root) = filled(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.history_retention_status(&request).unwrap();
    assert_eq!(before.pending_releases, 0);
    let narrow = WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::MIN,
        ..limits()
    };
    let cursor = document
        .history_page(&request, narrow, None)
        .unwrap()
        .continuation()
        .unwrap();
    let pause = document.pause_before_history_retention_change_for_test();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped = host.request(&principal, &stopped_scope);
    let (reached, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            reached
        });
        let result = document
            .set_history_window(ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN))
            .advance::<Window>(&stopped);
        (worker.join().unwrap(), result)
    });
    assert!(
        reached,
        "real READ, metadata and window admission preceded the pause"
    );
    cancelled(result);
    let after = document.history_retention_status(&request).unwrap();
    assert_eq!(
        (
            after.recent_steps,
            after.pending_window,
            after.unprotected_steps,
            after.current,
            after.pending_releases
        ),
        (
            before.recent_steps,
            None,
            before.unprotected_steps,
            before.current,
            0
        )
    );
    assert!(document
        .history_page(&request, narrow, Some(&cursor))
        .is_ok());
    document.preview(&request, root.id()).unwrap().close();
    window(&host, &mut document, 1);
    close(&host, document, 5);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: cancellation_after_one_window_batch_preserves_its_prefix_and_original_disposal_owners joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn cancellation_after_one_window_batch_preserves_its_prefix_and_original_disposal_owners() {
    let host = publish_on_first_program();
    let (mut document, root) = filled(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let preview = document.preview(&request, root.id()).unwrap();
    // An actual old-head-only successor is outside the selected future. Its
    // independent reader must keep that retired branch's real cleanup pending.
    let old_branch = root.endpoint().product_branch();
    assert_eq!(
        settle(set_retention(&host, old_branch, 9, 0x9182_6610)),
        RetentionVerdict::Performed(9)
    );
    let held = observe_head(host.runtime(), old_branch);
    let narrow = WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::MIN,
        ..limits()
    };
    let cursor = document
        .history_page(&request, narrow, None)
        .unwrap()
        .continuation()
        .unwrap();
    let pause = document.pause_after_history_retention_change_for_test();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped = host.request(&principal, &stopped_scope);
    let target = ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN);
    let (reached, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            reached
        });
        let result = document
            .set_history_window(target)
            .advance::<Window>(&stopped);
        (worker.join().unwrap(), result)
    });
    assert!(reached, "one real exclusion preceded the delivery pause");
    cancelled(result);
    let prefix = document.history_retention_status(&request).unwrap();
    assert_eq!(
        (
            prefix.pending_window,
            prefix.unprotected_steps,
            prefix.pending_releases
        ),
        (Some(target), 3, 1)
    );
    assert_eq!(document.selection(&request).unwrap(), before);
    assert!(matches!(
        document.history_page(&request, narrow, Some(&cursor)),
        Err(Failure::Document(Native::Paging(
            WorthQueryHistoryPageDenial::StaleCursor
        )))
    ));
    assert!(matches!(
        document.preview(&request, root.id()),
        Err(Failure::Document(Native::HistoryNodeExcluded))
    ));
    assert_eq!(
        preview.read(&document, &request).unwrap().rows()[0].retention_days,
        7
    );
    window(&host, &mut document, 1);
    assert_eq!(value(&host, &document), 4);
    assert!(
        document
            .history_retention_status(&request)
            .unwrap()
            .pending_releases
            > 0
    );
    preview.close();
    let failure = host
        .with_host_advancement(|phase| document.close(&phase).unwrap_err())
        .unwrap();
    let WorthQueryApplicationHistoryCloseFailure::Closing(closing) = failure else {
        panic!("the same queued old-head cleanup must remain owned");
    };
    assert!(matches!(
        closing.denial(),
        Some(WorthQueryDocumentHistoryClosingDenial::Cleanup(
            WorthQueryApplicationProductBranchCleanupDenial::Product(
                WorthQueryProductBranchOwnerCleanupDenial::WorldHistoryStillRetained
            )
        ))
    ));
    drop(held);
    let receipt = host
        .with_host_advancement(|phase| closing.retry(&phase).unwrap())
        .unwrap();
    assert_eq!(receipt.closed_heads(), 5);
    assert!(host.branches().pending_cleanup().is_empty());
}
