//! Changed-frontier visits vary independently from unrelated retained history.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::declaration::application_history::ApplicationHistoryRecentSteps;
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::retention_policy::Window;

#[test]
fn window_batches_visit_only_the_changed_frontier_and_admit_their_own_bounded_work() {
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut samples = Vec::new();
    for depth in [8, 24] {
        for excluded in [1, 3] {
            let mut document = filled(&host, depth);
            let before = document.selection(&request).unwrap();
            let observer = document.observe_history_enrollment_for_test();
            let target =
                ApplicationHistoryRecentSteps::new(NonZeroUsize::new(depth - excluded).unwrap());
            let mut change = document.set_history_window(target);
            let mut calls = 0;
            let final_status = loop {
                let status = change.advance::<Window>(&request).unwrap();
                calls += 1;
                if status.pending_window.is_none() {
                    break status;
                }
                assert!(
                    calls < excluded,
                    "each nonterminal batch excludes a real source"
                );
            };
            drop(change);
            let evidence = observer.snapshot();
            drop(observer);
            assert_eq!(calls, excluded);
            assert_eq!(evidence.retention_frontier_probes, excluded as u64);
            assert!(
                evidence.retention_quoted_work > 0,
                "READ work cannot substitute for controller work"
            );
            assert_eq!(final_status.unprotected_steps, depth - excluded);
            assert_eq!(document.selection(&request).unwrap(), before);
            samples.push((depth, excluded, evidence.retention_quoted_work));
            close(&host, document, depth + 1);
        }
    }
    for excluded in [1, 3] {
        let work = |depth| {
            samples
                .iter()
                .find(|sample| sample.0 == depth && sample.1 == excluded)
                .unwrap()
                .2
        };
        assert!(
            work(24) <= work(8) * 2,
            "balanced index work must not scan unrelated retained entries"
        );
    }
    for depth in [8, 24] {
        let work = |excluded| {
            samples
                .iter()
                .find(|sample| sample.0 == depth && sample.1 == excluded)
                .unwrap()
                .2
        };
        assert!(
            work(3) <= work(1) * 4,
            "three changed sources require bounded per-frontier work"
        );
    }
}
