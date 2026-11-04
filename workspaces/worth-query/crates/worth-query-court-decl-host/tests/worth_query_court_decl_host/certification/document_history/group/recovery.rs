//! Original unpublished disposal does not reauthorize business at an obsolete document head.
use super::*;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::WorthQuerySourceOnlyPosture,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    presented_request::set_retention,
    readback::read_retention,
    retention_entry::SetRetentionHandler,
    settled_verdict::{settle, RetentionVerdict},
};

#[test]
fn abort_settles_the_original_unpublished_member_even_after_the_source_head_diverges() {
    for move_original in [false, true] {
        let (host, contacts) =
            SetRetentionHandler::observe_installation_for_test(publish_on_first_program);
        let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let original = document.selection(&request).unwrap();
        let mut group = begin(&host, document, 5, 0x9182_5600);
        interrupt_first(&host, &mut group);
        let effect_counts = contacts.snapshot();
        assert!(effect_counts.0 > 0 && effect_counts.1 > 0);
        if move_original {
            assert_eq!(
                settle(set_retention(
                    &host,
                    original.physical_endpoint().product_branch(),
                    6,
                    0x9182_5601
                )),
                RetentionVerdict::Performed(6)
            );
            assert!(matches!(
                group.document().read(&request),
                Err(WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::HeadDiverged
                ))
            ));
        }
        let before_close = contacts.snapshot();
        let document = group.abort(&request).unwrap();
        assert_eq!(
            contacts.snapshot(),
            before_close,
            "original settlement must not reenter either domain stage"
        );
        assert!(host
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .is_empty());
        assert!(host.branches().pending_cleanup().is_empty());
        if move_original {
            assert_eq!(
                read_retention(&host, original.physical_endpoint().product_branch()),
                6
            );
        } else {
            assert_eq!(value(&host, &document), 7);
            assert_eq!(document.selection(&request).unwrap(), original);
        }
        close(&host, document, 1);
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-scope=1 scoped-thread-spawn=1: interrupt_first joins the bounded native owner and competing host or control actors to certify custody at the real handoff."
)]
fn interrupt_first(host: &App, group: &mut Group<'_>) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    until(host, group, |group| group.posture() == Posture::Running);
    {
        let (document, execution) = group.parts();
        execution.advance(document, &request).unwrap();
    }
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let interrupted_principal = authenticate_operator(host.installed_schema(), &stopped);
    let pause = host
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let reached = std::thread::scope(|threads| {
        let (document, execution) = group.parts();
        let worker = threads
            .spawn(|| execution.advance(document, &host.request(&interrupted_principal, &stopped)));
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        let _ = worker.join().unwrap();
        reached
    });
    assert!(
        reached,
        "actual native effect must precede failed World comparison"
    );
    {
        let (_, execution) = group.parts();
        assert_eq!(
            execution.source_operation(0).unwrap().posture(),
            WorthQuerySourceOnlyPosture::ProductUnpublished
        );
        assert!(execution.selected_step().is_none());
    }
    let partials = host
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(partials.rows().len(), 1);
    let partial = host
        .readmit_product_publication_recovery(partials.rows()[0].handle())
        .unwrap()
        .inspect()
        .unwrap();
    assert_eq!(partial.owner_effect_count(), 1);
    assert!(partial.live_obligation_count() > 0);
    drop(partial);
    drop(partials);
}

#[test]
fn an_unpublished_first_member_resumes_once_then_completes_the_one_group_step() {
    let (host, contacts) =
        SetRetentionHandler::observe_installation_for_test(publish_on_first_program);
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut group = begin(&host, document, 5, 0x9182_5610);
    interrupt_first(&host, &mut group);
    let before = contacts.snapshot();
    until(&host, &mut group, |group| group.completed_members() == 1);
    assert_eq!(
        contacts.snapshot(),
        before,
        "recovery must not replay the original handler"
    );
    {
        let (document, execution) = group.parts();
        assert_eq!(value(&host, document), 7);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            3
        );
        assert!(matches!(
            execution.completed_source(0).unwrap().evidence(),
            WorthQuerySourceCompletion::Recovered {
                publication: Some(_),
                ..
            }
        ));
    }
    until(&host, &mut group, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    let step = {
        let (document, execution) = group.parts();
        execution.complete(document, &request).unwrap()
    };
    assert_eq!(step.parent(), Some(root.id()));
    let finished = group.finish(&request).unwrap();
    assert_eq!(value(&host, &finished.document), 5);
    assert_eq!(finished.completions.len(), 2);
    drop(finished.completions);
    close(&host, finished.document, 2);
}
