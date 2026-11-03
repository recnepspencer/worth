//! Original request and selected-head guards decide whether assembled hints can leave.
use super::*;
use std::time::Duration;
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{WorthQueryCancellationSource, WorthQueryRequestScope},
    presented_request::set_retention,
    settled_verdict::{settle, RetentionVerdict},
};

#[derive(Clone, Copy, Debug)]
enum Change {
    Cancel,
    SelectChild,
    OutsideWrite,
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: assembled_redo_hints_cannot_escape_a_stop_or_move_and_a_cancelled_cursor_can_retry joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn assembled_redo_hints_cannot_escape_a_stop_or_move_and_a_cancelled_cursor_can_retry() {
    for change in [Change::Cancel, Change::SelectChild, Change::OutsideWrite] {
        let host = publish_on_first_program();
        let (mut document, children) = forked(&host);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let original = document.selection(&request).unwrap();
        let cursor = document
            .redo_choices(&request, limits(1), None)
            .unwrap()
            .children()
            .continuation()
            .unwrap();
        let cancel = WorthQueryCancellationSource::new();
        let worker_scope = WorthQueryRequestScope::new(None, cancel.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
        let pause = document.pause_after_history_page_assembly_for_test();
        let (reached, action, result) = std::thread::scope(|threads| {
            let worker = threads.spawn(|| {
                document.redo_choices(
                    &host.request(&principal, &worker_scope),
                    limits(1),
                    Some(&cursor),
                )
            });
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            let action = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert!(reached, "actual choice page assembly");
                match change {
                    Change::Cancel => cancel.cancel(),
                    Change::SelectChild => {
                        checkout(&host, &document, children[0].id(), 0x9182_f240)
                    }
                    Change::OutsideWrite => assert_eq!(
                        settle(set_retention(
                            &host,
                            original.physical_endpoint().product_branch(),
                            3,
                            0x9182_f241
                        )),
                        RetentionVerdict::Performed(3)
                    ),
                }
            }));
            pause.release();
            (reached, action, worker.join())
        });
        assert!(reached);
        action.unwrap();
        let denied = result.unwrap().unwrap_err();
        match change {
            Change::Cancel => {
                assert!(matches!(denied, Failure::Document(Native::Paging(PageFailure::Request(worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)))))
                    || matches!(denied, Failure::ExecutionRequest(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))), "{denied:?}");
                let retried = document.redo_choices(&request, limits(1), Some(&cursor)).unwrap();
                assert_eq!(retried.choice(), Choice::ChoiceRequired);
                assert_page_entry(retried.children().rows()[0].step(), &children[1]);
                assert_eq!(document.selection(&request).unwrap(), original);
            }
            Change::SelectChild => assert!(matches!(denied, Failure::Document(Native::HeadDiverged)), "{denied:?}"),
            Change::OutsideWrite => assert!(matches!(denied, Failure::Document(Native::NativeBasis(worth_relational::facade::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget))), "{denied:?}"),
        }
        close(
            &host,
            document,
            if matches!(change, Change::SelectChild) {
                6
            } else {
                5
            },
        );
    }
}
