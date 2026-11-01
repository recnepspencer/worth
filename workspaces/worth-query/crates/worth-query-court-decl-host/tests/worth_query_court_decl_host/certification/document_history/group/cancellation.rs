//! Cancellation before the next effect/adoption preserves the exact original group.
use super::*;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{WorthQueryDocumentHistoryDenial, WorthQueryHistoryPageLimits},
    product::WorthQueryExpectedProductBranchAdoptionDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::retention_entry::SetRetentionHandler;

fn cancelled<T>(result: Result<T, WorthQueryHistoryGroupDenial>) {
    let Err(cause) = result else {
        panic!("cancelled request cannot deliver success")
    };
    // A stopped request cannot begin the next member's advancement.
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::History(
                WorthQueryApplicationHistoryDenial::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                )
            )
        ),
        "{cause:?}"
    );
}

fn finish(host: &App, group: Group<'_>) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let finished = group.finish(&request).unwrap();
    assert_eq!(finished.completions.len(), 2);
    let limits = WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::new(3).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(3).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    };
    let page = finished
        .document
        .history_page(&request, limits, None)
        .unwrap();
    assert_eq!(page.rows().len(), 2);
    assert_page_entry(page.rows()[1].step(), &finished.step);
    assert_eq!(finished.step.parent(), Some(page.rows()[0].step().id()));
    assert_eq!(value(host, &finished.document), 5);
    drop(page);
    drop(finished.completions);
    close(host, finished.document, 2);
}

#[test]
fn cancellation_between_completed_members_preserves_the_first_publication_and_resumes_without_replay(
) {
    let (host, contacts) =
        SetRetentionHandler::observe_installation_for_test(publish_on_first_program);
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let mut group = begin(&host, document, 5, 0x9182_5c00);
    until(&host, &mut group, |group| group.completed_members() == 1);
    let first = {
        let (_, execution) = group.parts();
        let WorthQuerySourceCompletion::Fresh { receipt, .. } =
            execution.completed_source(0).unwrap().evidence()
        else {
            panic!("first original publication")
        };
        receipt.commit_reference().clone()
    };
    let one_effect = contacts.snapshot();
    assert_eq!(one_effect, (1, 1));
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let interrupted_principal = authenticate_operator(host.installed_schema(), &stopped);
    cancellation.cancel();
    {
        let (document, execution) = group.parts();
        cancelled(execution.advance(document, &host.request(&interrupted_principal, &stopped)));
        assert_eq!(execution.completed_members(), 1);
        assert!(execution.completed_source(1).is_none());
        assert!(execution.selected_step().is_none());
        assert_eq!(document.selection(&request).unwrap(), before);
        assert_eq!(value(&host, document), 7);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            3
        );
    }
    assert_eq!(group.posture(), Posture::Running);
    assert_eq!(
        contacts.snapshot(),
        one_effect,
        "the next handler did not run after cancellation"
    );
    until(&host, &mut group, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    {
        let (document, execution) = group.parts();
        let WorthQuerySourceCompletion::Fresh { receipt, .. } =
            execution.completed_source(1).unwrap().evidence()
        else {
            panic!("second original publication")
        };
        assert_eq!(receipt.commit_reference().branch_id, first.branch_id);
        assert_eq!(receipt.commit_reference().parents, [first.commit_id]);
        execution.complete(document, &request).unwrap();
    }
    assert_eq!(
        contacts.snapshot(),
        (2, 2),
        "one execution per original member"
    );
    finish(&host, group);
    assert_eq!(contacts.snapshot(), (2, 2));
}

#[test]
fn cancellation_after_endpoint_capture_preserves_the_ready_candidate_for_fresh_guarded_adoption() {
    let (host, contacts) =
        SetRetentionHandler::observe_installation_for_test(publish_on_first_program);
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let mut group = begin(&host, document, 5, 0x9182_5c10);
    until(&host, &mut group, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    let endpoint = {
        let (_, execution) = group.parts();
        let WorthQuerySourceCompletion::Fresh { receipt, .. } =
            execution.completed_source(1).unwrap().evidence()
        else {
            panic!("actual last member endpoint")
        };
        receipt
            .committed_product_publication()
            .composite_commit()
            .clone()
    };
    let effects = contacts.snapshot();
    assert_eq!(effects, (2, 2));
    let pause = group.pause_after_history_capture_for_test();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let interrupted_principal = authenticate_operator(host.installed_schema(), &stopped);
    let (reached, result) = std::thread::scope(|threads| {
        let (document, execution) = group.parts();
        let worker = threads.spawn(|| {
            execution.complete(document, &host.request(&interrupted_principal, &stopped))
        });
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        (reached, worker.join().unwrap())
    });
    assert!(
        reached,
        "the real captured endpoint must reach its original preadoption receiver"
    );
    assert!(
        matches!(
            result,
            Err(WorthQueryHistoryGroupDenial::History(
                WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::Adoption(
                        WorthQueryExpectedProductBranchAdoptionDenial::Stopped(
                            WorthQueryAdvancementDenial::Interrupted(
                                WorthQueryManagedComputationInterruption::Cancelled
                            )
                        )
                    )
                )
            ))
        ),
        "{result:?}"
    );
    assert_eq!(group.posture(), Posture::ReadyToComplete);
    assert_eq!(contacts.snapshot(), effects);
    {
        let (document, execution) = group.parts();
        assert!(execution.selected_step().is_none());
        assert_eq!(execution.completed_members(), 2);
        assert_eq!(document.selection(&request).unwrap(), before);
        assert_eq!(value(&host, document), 7);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            5
        );
        let step = execution.complete(document, &request).unwrap();
        assert_eq!(step.parent(), Some(before.step().id()));
        assert_eq!(step.endpoint().selected_commit(), &endpoint);
    }
    assert_eq!(group.posture(), Posture::Selected);
    assert_eq!(
        contacts.snapshot(),
        effects,
        "completion cannot replay either handler"
    );
    finish(&host, group);
    assert_eq!(contacts.snapshot(), effects);
}
