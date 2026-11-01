//! Completion and a competing navigation choose one exact physical document selection.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryHistoryNavigationAdvance;

fn reject_stale_navigation(
    host: &Runtime,
    mut owner: worth_query_host::facade::application_entry::WorthQueryHistoryNavigation<
        '_,
        '_,
        DocumentRetentionSchema,
        RetentionHistory,
        Navigation,
    >,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for _ in 0..16 {
        owner = match owner.advance(&request) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => next,
            WorthQueryHistoryNavigationAdvance::Refused { owner, cause } => {
                assert!(matches!(cause,worth_query_host::facade::application_entry::WorthQueryHistoryNavigationAdvanceDenial::Navigation(
                    worth_query_host::facade::application_entry::WorthQueryHistoryNavigationDenial::HeadChanged)),"{cause:?}");
                owner.discard(&request).unwrap();
                return;
            }
            WorthQueryHistoryNavigationAdvance::Selected(_) => {
                panic!("stale navigation cannot replace the completed group")
            }
        };
    }
    panic!("the stale navigation must reach its actual guard refusal");
}

#[test]
fn a_competing_navigation_and_group_completion_preserve_the_real_guard_winner() {
    for group_first in [false, true] {
        let host = publish_on_first_program();
        let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let root = document.current_step(&request).unwrap();
        let mut group = begin(&host, document, 5, 0x9182_5900);
        until(&host, &mut group, |group| {
            group.posture() == Posture::ReadyToComplete
        });
        let (document, execution) = group.parts();
        let navigation = document
            .navigate::<Navigation>()
            .checkout(root.id())
            .expected_selection(document.selection(&request).unwrap())
            .idempotency(0x9182_5901)
            .prepare(&request)
            .unwrap();
        let navigation = super::super::navigation::awaiting(&host, navigation);
        if group_first {
            let selected = execution.complete(document, &request).unwrap();
            reject_stale_navigation(&host, navigation);
            assert_eq!(document.current_step(&request).unwrap(), selected);
            assert_eq!(value(&host, document), 5);
            let finished = group.finish(&request).unwrap();
            drop(finished.completions);
            close(&host, finished.document, 2);
        } else {
            let moved = super::super::navigation::complete(&host, navigation);
            assert_eq!(moved.requested_node(), root.id());
            let cause = execution.complete(document, &request).unwrap_err();
            assert!(
                matches!(
                    cause,
                    WorthQueryHistoryGroupDenial::History(
                        WorthQueryApplicationHistoryDenial::Document(
                            WorthQueryDocumentHistoryDenial::HeadDiverged
                        )
                    )
                ),
                "{cause:?}"
            );
            assert_eq!(
                execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
                5
            );
            assert_eq!(value(&host, document), 7);
            let document = group.abort(&request).unwrap();
            assert_eq!(document.selection(&request).unwrap(), *moved.selection());
            close(&host, document, 2);
        }
    }
}
